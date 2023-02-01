use std::sync::Arc;

use futures::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicQosOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::{FieldTable, ShortUInt};
use lapin::ExchangeKind::Fanout;
use lapin::{Channel, Error};
use log::{error};
use tokio::sync::{Notify, RwLock};
use tokio::time::{sleep, Duration};

use crate::broker::Event;
use crate::conf::RMQ;
use crate::{Broker, Rmq};

const PREFETCH_COUNT: ShortUInt = 1;

pub struct Subscriber {
    rmq: Arc<RwLock<Rmq>>,
    broker: Arc<Broker>,
    conf: RMQ,
}

impl Subscriber {
    pub fn new(rmq: Arc<RwLock<Rmq>>, broker: Arc<Broker>, conf: RMQ) -> Self {
        Self { rmq, broker, conf }
    }

    pub async fn run(&self, notify: Arc<Notify>) -> Result<(), Error> {
        loop {
            if self.subscribe(notify.clone()).await? {
                break;
            }
        }

        Ok(())
    }

    pub async fn subscribe(&self, notify: Arc<Notify>) -> Result<bool, Error> {
        let ch = self.handle_connection().await;

        let queue_name = self.declare_request_queue(ch.clone()).await.unwrap();

        ch.basic_qos(PREFETCH_COUNT, BasicQosOptions::default()).await?;

        let mut consumer = ch
            .basic_consume(
                queue_name.as_str(),
                "",
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        loop {
            tokio::select! {
                Some(delivery) = consumer.next() => {
                    if let Ok(delivery) = delivery {
                        delivery.ack(BasicAckOptions { multiple: false }).await?;

                        self.broker.publish(Event::new(
                            delivery
                                .properties
                                .message_id()
                                .clone()
                                .unwrap()
                                .to_string(),
                            delivery.data,
                            delivery.properties.kind().clone().unwrap().to_string(),
                        ));
                    } else {
                        log::error!("consumer stopped: {:?}", delivery);

                        return Ok(false)
                    }
                }
                _ = notify.notified() => {
                    log::info!("subscriber received shutdown signal");

                    return Ok(true)
                }
            }
        }
    }

    async fn handle_connection(&self) -> Channel {
        loop {
            sleep(Duration::from_millis(250)).await;

            let rmq = self.rmq.read().await;

            let conn_wrapped = rmq.connect().await;

            if conn_wrapped.is_err() {
                error!(
                    "can't get connection from pool, {}",
                    conn_wrapped.unwrap_err()
                );

                continue;
            }

            let ch_wrapped = conn_wrapped.unwrap().create_channel().await;

            if ch_wrapped.is_err() {
                error!("can't create channel, {}", ch_wrapped.unwrap_err());

                continue;
            }

            return ch_wrapped.unwrap();
        }
    }

    async fn declare_request_queue(&self, rmq_ch: Channel) -> Result<String, Error> {
        rmq_ch
            .exchange_declare(
                self.conf.response_exchange.as_str(),
                Fanout,
                ExchangeDeclareOptions {
                    passive: false,
                    durable: true,
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        let queue = rmq_ch
            .queue_declare(
                "",
                QueueDeclareOptions {
                    passive: false,
                    durable: false,
                    exclusive: false,
                    auto_delete: true,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        rmq_ch
            .queue_bind(
                queue.name().as_str(),
                self.conf.response_exchange.as_str(),
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(queue.name().to_string())
    }
}
