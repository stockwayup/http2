#![deny(warnings)]
#![forbid(unsafe_code)]

use futures::SinkExt;
use json_env_logger2::builder;
use json_env_logger2::env_logger::Target;
use log::{warn, LevelFilter};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::conf::Conf;
use crate::routes::build_routes;
use crate::signals::listen_signals;

mod conf;
mod events;
mod handlers;
mod responses;
mod routes;
mod signals;

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

    let nats_client = Arc::new(RwLock::new(
        async_nats::ConnectOptions::new()
            .ping_interval(std::time::Duration::from_secs(10))
            .request_timeout(Some(std::time::Duration::from_secs(10)))
            .connect(conf.nats.host)
            .await?,
    ));

    let routes = build_routes(conf.allowed_origins, nats_client.clone());

    let notify = listen_signals();

    let server_shutdown_notify = notify.clone();

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), conf.listen_port);

    let server = axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .with_graceful_shutdown(async move {
            server_shutdown_notify.notified().await;

            log::info!("server received shutdown signal")
        });

    let result = tokio::try_join!(tokio::task::spawn(server),);

    match result {
        Ok(_) => log::info!("shutdown completed"),
        Err(e) => log::error!("thread join error {}", e),
    }

    nats_client.write().await.close().await?;

    Ok(())
}
