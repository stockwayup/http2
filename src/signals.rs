use std::sync::Arc;

use libc::{SIGINT, SIGTERM};
use tokio::signal::unix::SignalKind;
use tokio::sync::Notify;

pub fn listen_signals() -> Arc<Notify> {
    let notify = Arc::new(Notify::new());

    for &signum in [SIGTERM, SIGINT].iter() {
        let signal_result = tokio::signal::unix::signal(SignalKind::from_raw(signum));

        match signal_result {
            Ok(mut sig) => {
                let notify = notify.clone();

                tokio::spawn(async move {
                    sig.recv().await;

                    notify.notify_waiters();

                    log::info!("shutdown signal received");
                });
            }
            Err(e) => {
                log::error!(
                    "failed to register signal handler for signal {}: {}",
                    signum,
                    e
                );
                // Continue with other signals - don't crash the application
                continue;
            }
        }
    }

    log::info!("waiting for signal");

    notify
}
