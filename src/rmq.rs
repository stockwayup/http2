use deadpool_lapin::Pool;
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use lapin::{Channel, Error};
use log::error;

type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

const REQ_QUEUE: &str = "http.responses";
const RESP_QUEUE: &str = "http.requests";

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
            error!("can't create channel, {}", e);

            e
        })?;

        Ok(ch)
    }

    pub async fn declare_queues(&self, ch: Channel) -> Result<(), Error> {
        ch.queue_declare(
            REQ_QUEUE,
            QueueDeclareOptions {
                passive: false,
                durable: true,
                exclusive: false,
                auto_delete: false,
                nowait: false,
            },
            FieldTable::default(),
        )
        .await?;

        ch.queue_declare(
            RESP_QUEUE,
            QueueDeclareOptions {
                passive: false,
                durable: true,
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
