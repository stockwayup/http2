use std::time::Duration;

use deadpool::managed::Timeouts;
use deadpool_lapin::Pool;
use deadpool_lapin::{Manager, Runtime};
use kv_log_macro as log;
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::ConnectionProperties;
use lapin::{Channel, Error};

type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

const QUEUE: &str = "http.requests";

pub struct Rmq {
    pool: Pool,
    conn: Connection,
}

impl Rmq {
    pub async fn new(pool: Pool) -> Self {
        let conn = pool.get().await.unwrap();

        Self { pool, conn }
    }

    pub async fn open_ch(&self) -> Result<Channel, Box<dyn std::error::Error>> {
        let ch = self.conn.create_channel().await.map_err(|e| {
            log::error!("can't create channel, {}", e);

            e
        })?;

        Ok(ch)
    }

    pub async fn declare_queues(&self, ch: Channel) -> Result<(), Error> {
        ch.queue_declare(
            QUEUE,
            QueueDeclareOptions {
                passive: false,
                durable: false,
                exclusive: false,
                auto_delete: false,
                nowait: false,
            },
            FieldTable::default(),
        )
        .await?;

        Ok(())
    }

    pub fn close(&self) {
        self.pool.close();
    }
}

pub async fn setup_rmq() -> Rmq {
    let addr =
        std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://user:pass@127.0.0.1:5672/%2f".into());

    let manager = Manager::new(
        addr,
        ConnectionProperties::default()
            .with_executor(tokio_executor_trait::Tokio::current())
            .with_reactor(tokio_reactor_trait::Tokio),
    );

    let pool = deadpool::managed::Pool::builder(manager)
        .runtime(Runtime::Tokio1)
        .max_size(10)
        .timeouts(Timeouts {
            wait: Some(Duration::new(5, 0)),
            create: Some(Duration::new(5, 0)),
            recycle: Some(Duration::new(300, 0)),
        })
        .build()
        .expect("can't create pool");

    let rmq = Rmq::new(pool.clone()).await;

    let ch = rmq.open_ch().await.unwrap();

    rmq.declare_queues(ch).await.unwrap();

    rmq
}
