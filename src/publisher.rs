use std::ops::Add;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use kv_log_macro as log;
use lapin::options::BasicPublishOptions;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::{ShortShortUInt, ShortString};
use lapin::Channel;
use rmp_serde::Serializer;
use serde::Serialize;
use uuid::Uuid;

use crate::events::HttpReq;

const QUEUE: &str = "http.requests";

pub struct Publisher {
    rmq_ch: Channel,
}

impl Publisher {
    pub fn new(ch: Channel) -> Self {
        Self { rmq_ch: ch }
    }

    pub async fn publish<'b, T: serde::Serialize>(
        &self,
        req: HttpReq<'b, T>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();

        let mut se = Serializer::new(&mut buf).with_struct_map();

        req.serialize(&mut se).unwrap();

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let expiration = since_the_epoch
            .add(Duration::from_secs(120))
            .as_secs()
            .to_string();

        let id = Uuid::new_v4();

        let props = AMQPProperties::default()
            .with_content_type(ShortString::from("application/octet-stream"))
            .with_message_id(ShortString::from(id.to_string()))
            .with_delivery_mode(ShortShortUInt::from(1))
            .with_expiration(ShortString::from(expiration));

        self.rmq_ch
            .basic_publish(
                "",
                QUEUE,
                BasicPublishOptions::default(),
                buf.as_slice(),
                props,
            )
            .await
            .map_err(|e| {
                log::error!("can't publish: {}", e);

                e
            })?;

        Ok(id.to_string())
    }
}
