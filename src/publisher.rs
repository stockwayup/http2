use std::ops::Add;
use std::result::Result as StdResult;
use std::time::Duration;

use deadpool_lapin::{Pool, PoolError};
use lapin::options::BasicPublishOptions;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::{ShortShortUInt, ShortString};
use log::error;
use rmp_serde::Serializer;
use serde::Serialize;
use uuid::Uuid;

use crate::events::HttpReq;

type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;
type RMQResult<T> = StdResult<T, PoolError>;

pub struct Publisher {
    pub pool: Pool,
}

impl<'a> Publisher {
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
        }
    }

    pub async fn publish<'b, T: serde::Serialize>(&self, req: HttpReq<'b, T>) -> Result<(), Box<dyn std::error::Error>> {
        let rmq_con = self.get_rmq_con(&self.pool).await.map_err(|e| {
            error!("can't connect to rmq, {}", e);

            e
        })?;

        let channel = rmq_con.create_channel().await.map_err(|e| {
            error!("can't create channel, {}", e);

            e
        })?;

        let mut buf = Vec::new();

        let mut se = Serializer::new(&mut buf).with_struct_map();

        req.serialize(&mut se).unwrap();

        use std::time::{SystemTime, UNIX_EPOCH};

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let expiration = since_the_epoch.add(Duration::from_secs(120)).as_secs().to_string();

        let props = AMQPProperties::default().
            with_content_type(ShortString::from("application/octet-stream")).
            with_message_id(ShortString::from(Uuid::new_v4().to_string())).
            with_delivery_mode(ShortShortUInt::from(1)).
            with_expiration(ShortString::from(expiration))
            ;

        channel
            .basic_publish(
                "",
                "http.requests",
                BasicPublishOptions::default(),
                buf.as_slice(),
                props,
            )
            .await
            .map_err(|e| {
                error!("can't publish: {}", e);

                e
            })?;

        Ok(())
    }

    async fn get_rmq_con(&self, pool: &Pool) -> RMQResult<Connection> {
        let connection = pool.get().await?;

        Ok(connection)
    }
}
