// #![deny(warnings)]
#![forbid(unsafe_code)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use json_env_logger2::builder;
use json_env_logger2::env_logger::Target;
use log::LevelFilter;
use tokio::sync::RwLock;

use crate::broker::Broker;
use crate::conf::Conf;
use crate::publisher::Publisher;
use crate::rmq::setup_rmq;
use crate::routes::build_routes;
use crate::signals::listen_signals;
use crate::subscriber::Subscriber;

mod broker;
mod conf;
mod events;
mod handlers;
mod publisher;
mod responses;
mod rmq;
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

    let conf = Conf::new();

    let rmq = setup_rmq().await;

    let rmq_ch = rmq.open_ch().await.unwrap();

    let pub_svc = Arc::new(RwLock::new(Publisher::new(rmq_ch)));

    let rmq_ch = rmq.open_ch().await.unwrap();

    let broker = Arc::new(Broker::new());

    let sub_svc = Subscriber::new(rmq_ch, broker.clone());

    let routes = build_routes(pub_svc.clone(), broker.clone());

    let notify = listen_signals();

    let server_shutdown_notify = notify.clone();
    let router_shutdown_notify = notify.clone();
    let sub_svc_shutdown_notify = notify.clone();

    let addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        conf.unwrap().listen_port,
    );

    let server = axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .with_graceful_shutdown(async move {
            server_shutdown_notify.notified().await;

            log::info!("server received shutdown signal")
        });

    let result = tokio::try_join!(
        tokio::task::spawn(server),
        tokio::task::spawn(async move { broker.run(router_shutdown_notify).await }),
        tokio::task::spawn(async move { sub_svc.subscribe(sub_svc_shutdown_notify).await }),
    );

    match result {
        Ok(_) => log::info!("shutdown completed"),
        Err(e) => log::error!("thread join error {}", e),
    }

    rmq.close();

    Ok(())
}
