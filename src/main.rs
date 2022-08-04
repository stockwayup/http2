// #![deny(warnings)]
#![forbid(unsafe_code)]

use std::sync::Arc;
use std::time::Duration;

use deadpool::managed::Timeouts;
use deadpool_lapin::{Manager, Runtime};
use json_env_logger2::builder;
use json_env_logger2::env_logger::Target;
use lapin::ConnectionProperties;
use libc::{c_int, SIGINT, SIGTERM};
use log::LevelFilter;
use tokio::signal::unix::SignalKind;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, RwLock};

use crate::publisher::Publisher;
use crate::rmq::Rmq;
use crate::router::Router;
use crate::routes::build_routes;
use crate::subscriber::Subscriber;

mod events;
mod handlers;
mod publisher;
mod responses;
mod rmq;
mod router;
mod routes;
mod subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = builder();

    builder.target(Target::Stdout);
    builder.filter_level(LevelFilter::Debug);
    builder.try_init().unwrap();

    json_env_logger2::panic_hook();

    let rmq = setup_rmq().await;

    let rmq_ch = rmq.open_ch().await.unwrap();

    let pub_svc = Arc::new(RwLock::new(Publisher::new(rmq_ch)));

    let rmq_ch = rmq.open_ch().await.unwrap();

    let router = Arc::new(Router::new());

    let sub_svc = Arc::new(RwLock::new(Subscriber::new(rmq_ch, router.clone())));

    let routes = build_routes(pub_svc.clone(), router.clone());

    let mut rx = listen_signals();

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 8000), async move {
            log::info!(target: "app", "waiting for signal");
            rx.recv().await;
            log::info!(target: "app", "shutdown signal received");
        });

    let result = tokio::try_join!(
        tokio::task::spawn(server),
        tokio::task::spawn(async move {
            router.run().await;
        }),
        tokio::task::spawn(async move {
            sub_svc.read().await.subscribe().await;
        }),
    );

    match result {
        Ok(_) => log::info!(target: "app", "run tasks"),
        Err(e) => log::error!(target: "app", "thread join error {}", e),
    }

    rmq.close();

    Ok(())
}

async fn setup_rmq() -> Rmq {
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

fn listen_signals() -> Receiver<c_int> {
    let (tx, rx) = mpsc::channel(2);

    for &signum in [SIGTERM, SIGINT].iter() {
        let tx = tx.clone();

        let mut sig = tokio::signal::unix::signal(SignalKind::from_raw(signum)).unwrap();

        tokio::spawn(async move {
            loop {
                sig.recv().await;
                if tx.clone().send(signum).await.is_err() {
                    break;
                };
            }
        });
    }

    rx
}
