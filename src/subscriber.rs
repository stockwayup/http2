use std::sync::Arc;

use futures::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicQosOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::{FieldTable, ShortUInt};
use lapin::ExchangeKind::Fanout;
use lapin::{Channel, Error};

use crate::router::Event;
use crate::Router;

const RESP_EXCHANGE: &str = "http.responses";
const PREFETCH_COUNT: ShortUInt = 1;

pub struct Subscriber {
    rmq_ch: Channel,
    router: Arc<Router>,
}

impl Subscriber {
    pub fn new(rmq_ch: Channel, router: Arc<Router>) -> Self {
        Self { rmq_ch, router }
    }

    pub async fn subscribe(&self) -> Result<(), Error> {
        let queue_name = self.declare_queues().await.unwrap();

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

        while let Some(delivery) = consumer.next().await {
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

        Ok(())
    }

    pub async fn declare_queues(&self) -> Result<String, Error> {
        self.rmq_ch
            .exchange_declare(
                RESP_EXCHANGE,
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
                RESP_EXCHANGE,
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(queue.name().to_string())
    }
}
