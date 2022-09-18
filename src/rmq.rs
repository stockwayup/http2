use std::time::Duration;

use deadpool::managed::Timeouts;
use deadpool_lapin::Pool;
use deadpool_lapin::{Manager, Runtime};
use kv_log_macro as log;
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::ConnectionProperties;
use lapin::{Channel, Error};

use crate::conf::RMQ;

type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

pub struct Rmq {
    pool: Pool,
    conn: Connection,
    conf: RMQ,
}

impl Rmq {
    pub async fn new(pool: Pool, conf: RMQ) -> Self {
        let conn = pool.get().await.expect("can't get connection from pool");

        Self { pool, conn, conf }
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
            self.conf.request_queue.as_str(),
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

pub async fn setup_rmq(conf: RMQ) -> Rmq {
    let addr = format!(
        "amqp://{}:{}@{}:{}/%2f",
        conf.user, conf.password, conf.host, conf.port
    );

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
        .expect("can't create rmq pool");

    let rmq = Rmq::new(pool.clone(), conf).await;

    let ch = rmq.open_ch().await.expect("can't open channel");

    rmq.declare_queues(ch).await.expect("can't declare queues");

    rmq
}
