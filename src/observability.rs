use crate::conf::Conf;
use crate::metrics::AppMetrics;
use std::sync::Arc;
use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_observability() -> Result<Arc<AppMetrics>, Box<dyn std::error::Error>> {
    // Try to load config early to determine log level
    let log_level = match Conf::new() {
        Ok(conf) => {
            if conf.is_debug {
                LevelFilter::DEBUG
            } else {
                LevelFilter::INFO
            }
        }
        Err(_) => LevelFilter::INFO, // Default to INFO if config fails
    };

    let subscriber_result = tracing_subscriber::registry()
        .with(log_level)
        .with(
            fmt::layer()
                .json()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_line_number(false)
                .with_current_span(false)
                .with_span_list(false),
        )
        .try_init();

    // Handle the case where subscriber is already initialized (graceful fallback)
    match subscriber_result {
        Ok(_) => println!("Tracing subscriber initialized successfully"),
        Err(e) => {
            println!("WARNING: Tracing subscriber already initialized: {}", e);
            eprintln!("WARNING: Tracing subscriber already initialized: {}", e);
            // Continue anyway - this is not fatal
        }
    }

    // Initialize Prometheus metrics with fallback
    let metrics = match AppMetrics::new() {
        Ok(metrics) => {
            println!("Prometheus metrics initialized successfully");
            Arc::new(metrics)
        }
        Err(e) => {
            println!("ERROR: Failed to initialize metrics: {}", e);
            eprintln!("ERROR: Failed to initialize metrics: {}", e);
            println!("Continuing without metrics collection (graceful degradation)");
            // For now, fail fast - but could be changed to graceful degradation
            return Err(e);
        }
    };

    println!("Observability system initialized");
    Ok(metrics)
}

pub fn shutdown_observability() {
    // Simplified shutdown for now
}
