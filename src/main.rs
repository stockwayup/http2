#![deny(warnings)]
#![forbid(unsafe_code)]

use futures::SinkExt;
use log::warn;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::conf::Conf;
use crate::observability::{init_observability, shutdown_observability};
use crate::routes::build_routes;
use crate::signals::listen_signals;

mod conf;
mod events;
mod handlers;
mod metrics;
mod observability;
mod responses;
mod routes;
mod signals;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Removed json_env_logger2::panic_hook() to avoid logger conflicts
    // Panic handling is now done by tracing_subscriber

    // Initialize observability (tracing and metrics) - this replaces json_env_logger2
    let metrics = match init_observability() {
        Ok(metrics) => Some(metrics),
        Err(e) => {
            // Use println! for early error logging since tracing may not be initialized
            println!("ERROR: Failed to initialize observability: {}", e);
            eprintln!("ERROR: Failed to initialize observability: {}", e);
            None
        }
    };

    let conf = match Conf::new() {
        Ok(conf) => conf,
        Err(err) => {
            warn!("failed to load configuration, {}", err);

            std::process::exit(1);
        }
    };

    // Log level is now controlled by tracing_subscriber in init_observability()
    if conf.is_debug {
        println!("DEBUG: Debug mode enabled");
    }

    let nats_client = Arc::new(RwLock::new(
        async_nats::ConnectOptions::new()
            .ping_interval(std::time::Duration::from_secs(10))
            .request_timeout(Some(std::time::Duration::from_secs(10)))
            .connect(conf.nats.host)
            .await?,
    ));

    let routes = build_routes(conf.allowed_origins, conf.enable_cors, nats_client.clone(), metrics.clone());

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

    // Shutdown observability
    shutdown_observability();

    Ok(())
}
