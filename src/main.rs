// #![deny(warnings)]
#![forbid(unsafe_code)]

use std::sync::Arc;

use json_env_logger2::builder;
use json_env_logger2::env_logger::Target;
use log::{info, LevelFilter};
use tokio::sync::RwLock;

use crate::publisher::Publisher;
use crate::rmq::setup_rmq;
use crate::router::Router;
use crate::routes::build_routes;
use crate::signals::listen_signals;
use crate::subscriber::Subscriber;

mod events;
mod handlers;
mod publisher;
mod responses;
mod rmq;
mod router;
mod routes;
mod signals;
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

    let sub_svc = Subscriber::new(rmq_ch, router.clone());

    let routes = build_routes(pub_svc.clone(), router.clone());

    let notify = listen_signals();

    let server_shutdown_notify = notify.clone();
    let router_shutdown_notify = notify.clone();
    let sub_svc_shutdown_notify = notify.clone();

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 8000), async move {
            server_shutdown_notify.notified().await;

            log::info!("server received shutdown signal")
        });

    let result = tokio::try_join!(
        tokio::task::spawn(server),
        tokio::task::spawn(async move { router.run(router_shutdown_notify).await }),
        tokio::task::spawn(async move { sub_svc.subscribe(sub_svc_shutdown_notify).await }),
    );

    match result {
        Ok(_) => log::info!("shutdown completed"),
        Err(e) => log::error!("thread join error {}", e),
    }

    rmq.close();

    Ok(())
}
