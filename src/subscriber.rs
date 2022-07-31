use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use lapin::{Channel, Error};

const QUEUE: &str = "http.responses";

pub struct Subscriber {
    rmq_ch: Channel,
}

impl Subscriber {
    pub fn new(ch: Channel) -> Self {
        Self { rmq_ch: ch }
    }

    pub async fn subscribe(&self) -> Result<(), Error> {
        let mut consumer = self
            .rmq_ch
            .basic_consume(
                QUEUE,
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
                println!("received msg: {:?}", delivery); //todo delete

                delivery.ack(BasicAckOptions { multiple: false }).await?

                // todo: publish
            }
        }

        Ok(())
    }
}
