use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use kv_log_macro as log;
use lapin::options::BasicPublishOptions;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::{ShortShortUInt, ShortString};
use lapin::Channel;
use rmp_serde::Serializer;
use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::events::HttpReq;
use crate::rmq::Rmq;

const QUEUE: &str = "http.requests";

pub struct Publisher {
    rmq: Arc<RwLock<Rmq>>,
    rmq_ch: Option<Channel>,
}

impl Publisher {
    pub fn new(rmq: Arc<RwLock<Rmq>>) -> Self {
        Self { rmq, rmq_ch: None }
    }

    pub async fn publish<'b>(
        &mut self,
        req: HttpReq<'b>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();

        let mut se = Serializer::new(&mut buf).with_struct_map();

        req.serialize(&mut se).unwrap();

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards");

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

        if self.rmq_ch.is_none() || !self.rmq_ch.as_ref().unwrap().status().connected() {
            {
                let rmq = self.rmq.read().await;

                let conn = rmq.connect().await?;

                self.rmq_ch = Some(rmq.open_ch(conn).await?);
            }
        }

        self.rmq_ch
            .as_ref()
            .unwrap()
            .basic_publish(
                "",
                QUEUE,
                BasicPublishOptions::default(),
                buf.as_slice(),
                props,
            )
            .await
            .map_err(|e| {
                log::error!("can't publish event: {}", e);

                e
            })?;

        Ok(id.to_string())
    }
}
