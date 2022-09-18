use std::sync::Arc;

use futures::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicQosOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::{FieldTable, ShortUInt};
use lapin::ExchangeKind::Fanout;
use lapin::{Channel, Error};
use tokio::sync::Notify;

use crate::broker::Event;
use crate::conf::RMQ;
use crate::Broker;

const PREFETCH_COUNT: ShortUInt = 1;

pub struct Subscriber {
    rmq_ch: Channel,
    router: Arc<Broker>,
    conf: RMQ,
}

impl Subscriber {
    pub fn new(rmq_ch: Channel, router: Arc<Broker>, conf: RMQ) -> Self {
        Self {
            rmq_ch,
            router,
            conf,
        }
    }

    pub async fn subscribe(&self, notify: Arc<Notify>) -> Result<(), Error> {
        // todo: redeclare queues after reconnect
        let queue_name = self.declare_request_queue().await.unwrap();

        self.rmq_ch
            .basic_qos(PREFETCH_COUNT, BasicQosOptions::default())
            .await?;

        let mut consumer = self
            .rmq_ch
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

                        self.router.publish(Event::new(
                            delivery
                                .properties
                                .message_id()
                                .clone()
                                .unwrap()
                                .to_string(),
                            delivery.data,
                            delivery.properties.kind().clone().unwrap().to_string(),
                        ));
                    }
                }
                _ = notify.notified() => {
                    log::info!("subscriber received shutdown signal");

                    break
                }
            }
        }

        Ok(())
    }

    pub async fn declare_request_queue(&self) -> Result<String, Error> {
        self.rmq_ch
            .exchange_declare(
                self.conf.response_exchange.as_str(),
                Fanout,
                ExchangeDeclareOptions {
                    passive: false,
                    durable: false,
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        let queue = self
            .rmq_ch
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

        self.rmq_ch
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
