// #![deny(warnings)]
#![forbid(unsafe_code)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use json_env_logger2::builder;
use json_env_logger2::env_logger::Target;
use log::{warn, LevelFilter};
use tokio::sync::RwLock;

use crate::broker::Broker;
use crate::conf::Conf;
use crate::publisher::Publisher;
use crate::rmq::{setup_rmq, Rmq};
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
    json_env_logger2::panic_hook();

    let mut builder = builder();

    builder.target(Target::Stdout);
    builder.filter_level(LevelFilter::Debug);
    builder.try_init().unwrap();

    let conf = match Conf::new() {
        Ok(conf) => conf,
        Err(err) => {
            warn!("failed to load configuration, {}", err);

            std::process::exit(1);
        }
    };

    if !conf.is_debug {
        log::set_max_level(LevelFilter::Info);
    }

    let rmq = Arc::new(RwLock::new(setup_rmq(conf.rmq.clone()).await));

    let pub_svc = Arc::new(RwLock::new(Publisher::new(rmq.clone())));

    let broker = Arc::new(Broker::new());

    let sub_svc = Subscriber::new(rmq.clone(), broker.clone(), conf.rmq);

    let routes = build_routes(conf.allowed_origins, pub_svc.clone(), broker.clone());

    let notify = listen_signals();

    let server_shutdown_notify = notify.clone();
    let router_shutdown_notify = notify.clone();
    let sub_svc_shutdown_notify = notify.clone();

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), conf.listen_port);

    let server = axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .with_graceful_shutdown(async move {
            server_shutdown_notify.notified().await;

            log::info!("server received shutdown signal")
        });

    let result = tokio::try_join!(
        tokio::task::spawn(server),
        tokio::task::spawn(async move { broker.run(router_shutdown_notify).await }),
        tokio::task::spawn(async move { sub_svc.run(sub_svc_shutdown_notify).await }),
    );

    match result {
        Ok(_) => log::info!("shutdown completed"),
        Err(e) => log::error!("thread join error {}", e),
    }

    rmq.read().await.close();

    Ok(())
}
