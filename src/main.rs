// #![deny(warnings)]
#![recursion_limit = "192"]

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use deadpool::managed::Timeouts;
use deadpool_lapin::{Manager, Runtime};
use env_logger::{Builder, Target};
use lapin::ConnectionProperties;
use libc::{c_int, SIGINT, SIGTERM};
use log::LevelFilter;
use log::{error, info};
use tokio::signal::unix::SignalKind;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, RwLock};
use warp::{route, Filter};

use crate::publisher::Publisher;
use crate::rmq::Rmq;
use crate::router::Router;
use crate::subscriber::Subscriber;

mod events;
mod handlers;
mod publisher;
mod responses;
mod rmq;
mod router;
mod subscriber;

const BODY_SIZE: u64 = 1024 * 250;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();

    builder.target(Target::Stdout);
    builder.filter_level(LevelFilter::Debug);
    builder.init();

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

    let ch = rmq.open_ch().await.unwrap();

    let pub_svc = Arc::new(RwLock::new(Publisher::new(ch)));

    let ch = rmq.open_ch().await.unwrap();

    let sub_svc = Arc::new(RwLock::new(Subscriber::new(ch)));

    let router = Arc::new(RwLock::new(Router::new()));

    let filters = warp::any()
        .and(route())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::query::<HashMap<String, String>>())
        .and(with_publisher(Arc::clone(&pub_svc)));

    let body_filters = filters.clone().and(warp::body::bytes());

    let get = warp::get().and(
        warp::path!("api" / "v1" / "statuses")
            .and_then(handlers::health_check)
            .or(warp::path!("api" / "v1" / "users" / u64 / "news")
                .and(filters.clone())
                .and(with_path_name("/api/v1/users/{uid}/news".to_string()))
                .and(with_param("uid".to_string()))
                .and_then(handlers::with_param))
            .or(warp::path!("api" / "v1" / "users" / u64 / "earnings")
                .and(filters.clone())
                .and(with_path_name("/api/v1/users/{uid}/earning".to_string()))
                .and(with_param("uid".to_string()))
                .and_then(handlers::with_param))
            .or(warp::path!("api" / "v1" / "users" / u64 / "dividends")
                .and(filters.clone())
                .and(with_path_name("/api/v1/users/{uid}/dividends".to_string()))
                .and(with_param("uid".to_string()))
                .and_then(handlers::with_param))
            .or(warp::path!("api" / "v1" / "users" / u64 / "day-prices")
                .and(filters.clone())
                .and(with_path_name("/api/v1/users/{uid}/day-prices".to_string()))
                .and(with_param("uid".to_string()))
                .and_then(handlers::with_param))
            .or(
                warp::path!("api" / "v1" / "users" / u64 / "day-price-periods")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/users/{uid}/day-prices-periods".to_string(),
                    ))
                    .and(with_param("uid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(warp::path!("api" / "v1" / "users" / u64 / "view-history")
                .and(filters.clone())
                .and(with_path_name(
                    "/api/v1/users/{uid}/view-history".to_string(),
                ))
                .and(with_param("uid".to_string()))
                .and_then(handlers::with_param))
            .or(warp::path!("api" / "v1" / "users" / u64)
                .and(filters.clone())
                .and(with_path_name("/api/v1/users/{uid}".to_string()))
                .and(with_param("uid".to_string()))
                .and_then(handlers::with_param))
            .or(warp::path!("api" / "v1" / "confirmation-codes")
                .and(filters.clone())
                .and(with_path_name("/api/v1/confirmation-codes".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "plans")
                .and(filters.clone())
                .and(with_path_name("/api/v1/plans".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "countries")
                .and(filters.clone())
                .and(with_path_name("/api/v1/countries".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "currencies")
                .and(filters.clone())
                .and(with_path_name("/api/v1/currencies".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "sectors")
                .and(filters.clone())
                .and(with_path_name("/api/v1/sectors".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "industries")
                .and(filters.clone())
                .and(with_path_name("/api/v1/industries".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "exchanges")
                .and(filters.clone())
                .and(with_path_name("/api/v1/exchanges".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "portfolios")
                .and(filters.clone())
                .and(with_path_name("/api/v1/portfolios".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "portfolios" / u64)
                .and(filters.clone())
                .and(with_path_name("/api/v1/portfolios/{pid}".to_string()))
                .and(with_param("pid".to_string()))
                .and_then(handlers::with_param))
            .or(warp::path!(
                "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions"
            )
            .and(filters.clone())
            .and(with_path_name(
                "/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string(),
            ))
            .and(with_param("pid".to_string()))
            .and(with_param("sid".to_string()))
            .and_then(handlers::with_2_params))
            .or(warp::path!(
                "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64
            )
            .and(filters.clone())
            .and(with_path_name(
                "/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string(),
            ))
            .and(with_param("pid".to_string()))
            .and(with_param("sid".to_string()))
            .and(with_param("tid".to_string()))
            .and_then(handlers::with_3_params))
            .or(
                warp::path!("api" / "v1" / "portfolios" / u64 / "securities")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/portfolios/{pid}/securities".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(warp::path!("api" / "v1" / "portfolios" / u64 / "news")
                .and(filters.clone())
                .and(with_path_name("/api/v1/portfolios/{pid}/news".to_string()))
                .and(with_param("pid".to_string()))
                .and_then(handlers::with_param))
            .or(warp::path!("api" / "v1" / "portfolios" / u64 / "earnings")
                .and(filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/earnings".to_string(),
                ))
                .and(with_param("pid".to_string()))
                .and_then(handlers::with_param))
            .or(warp::path!("api" / "v1" / "portfolios" / u64 / "dividends")
                .and(filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/dividends".to_string(),
                ))
                .and(with_param("pid".to_string()))
                .and_then(handlers::with_param))
            .or(
                warp::path!("api" / "v1" / "portfolios" / u64 / "day-prices")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/portfolios/{pid}/day-prices".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(
                warp::path!("api" / "v1" / "portfolios" / u64 / "day-price-periods")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/portfolios/{pid}/day-price-periods".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(warp::path!("api" / "v1" / "securities")
                .and(filters.clone())
                .and(with_path_name("/api/v1/securities".to_string()))
                .and_then(handlers::handle))
            .or(warp::path!("api" / "v1" / "securities" / u64 / "news")
                .and(filters.clone())
                .and(with_path_name("/api/v1/securities/{sid}/news".to_string()))
                .and(with_param("pid".to_string()))
                .and_then(handlers::with_param))
            .or(
                warp::path!("api" / "v1" / "securities" / u64 / "day-prices")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/securities/{sid}/day-prices".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(
                warp::path!("api" / "v1" / "securities" / u64 / "day-price-periods")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/securities/{sid}/day-price-periods".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(
                warp::path!("api" / "v1" / "securities" / u64 / "quarterly-balance-sheet")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/securities/{sid}/quarterly-balance-sheet".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(
                warp::path!("api" / "v1" / "securities" / u64 / "annual-balance-sheet")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/securities/{sid}/quarterly-balance-sheet".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(
                warp::path!("api" / "v1" / "securities" / u64 / "quarterly-income-statements")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/securities/{sid}/quarterly-income-statements".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(
                warp::path!("api" / "v1" / "securities" / u64 / "annual-income-statements")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/securities/{sid}/annual-income-statements".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(warp::path!("api" / "v1" / "securities" / u64)
                .and(filters.clone())
                .and(with_path_name("/api/v1/securities/{sid}".to_string()))
                .and(with_param("pid".to_string()))
                .and_then(handlers::with_param)),
    );

    let post = warp::post()
        .and(warp::body::content_length_limit(BODY_SIZE))
        .and(
            warp::path!("api" / "v1" / "users")
                .and(body_filters.clone())
                .and(with_path_name("/api/v1/users".to_string()))
                .and_then(handlers::with_body)
                .or(warp::path!("api" / "v1" / "refresh-tokens")
                    .and(body_filters.clone())
                    .and(with_path_name("/api/v1/refresh-tokens".to_string()))
                    .and_then(handlers::with_body))
                .or(warp::path!("api" / "v1" / "sessions")
                    .and(body_filters.clone())
                    .and(with_path_name("/api/v1/sessions".to_string()))
                    .and_then(handlers::with_body))
                .or(warp::path!("api" / "v1" / "confirmation-codes" / String)
                    .and(body_filters.clone())
                    .and(with_router(Arc::clone(&router)))
                    .and(with_path_name(
                        "/api/v1/confirmation-codes/{id}".to_string(),
                    ))
                    .and(with_param("id".to_string()))
                    .and_then(handlers::with_body_and_param))
                .or(warp::path!("api" / "v1" / "password-confirmation-codes")
                    .and(body_filters.clone())
                    .and(with_path_name(
                        "/api/v1/password-confirmation-codes".to_string(),
                    ))
                    .and_then(handlers::with_body))
                .or(warp::path!("api" / "v1" / "portfolios")
                    .and(body_filters.clone())
                    .and(with_path_name("/api/v1/portfolios".to_string()))
                    .and_then(handlers::with_body))
                .or(
                    warp::path!("api" / "v1" / "portfolios" / u64 / "relationships" / "securities")
                        .and(body_filters.clone())
                        .and(with_router(Arc::clone(&router)))
                        .and(with_path_name(
                            "/api/v1/portfolios/{pid}/relationships/securities".to_string(),
                        ))
                        .and(with_param("pid".to_string()))
                        .and_then(handlers::with_body_and_param),
                )
                .or(warp::path!(
                    "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions"
                )
                .and(body_filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string(),
                ))
                .and(with_param("pid".to_string()))
                .and(with_param("sid".to_string()))
                .and_then(handlers::with_body_and_2_params)),
        );

    let patch = warp::patch()
        .and(warp::body::content_length_limit(BODY_SIZE))
        .and(
            warp::path!("api" / "v1" / "portfolios" / u64)
                .and(body_filters.clone())
                .and(with_router(Arc::clone(&router)))
                .and(with_path_name("/api/v1/portfolios/{pid}".to_string()))
                .and(with_param("pid".to_string()))
                .and_then(handlers::with_body_and_param)
                .or(warp::path!(
                    "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64
                )
                .and(body_filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/securities/{sid}/transactions/{tid}".to_string(),
                ))
                .and(with_param("pid".to_string()))
                .and(with_param("sid".to_string()))
                .and(with_param("tid".to_string()))
                .and_then(handlers::with_body_and_3_params)),
        );

    let delete = warp::delete().and(
        warp::path!("api" / "v1" / "refresh-tokens" / u64)
            .and(filters.clone())
            .and(with_path_name(
                "/api/v1/refresh-tokens/{refresh-token}".to_string(),
            ))
            .and(with_param("refresh-token".to_string()))
            .and_then(handlers::with_param)
            .or(warp::path!("api" / "v1" / "portfolios" / u64)
                .and(filters.clone())
                .and(with_path_name("/api/v1/portfolios/{pid}".to_string()))
                .and(with_param("pid".to_string()))
                .and_then(handlers::with_param))
            .or(
                warp::path!("api" / "v1" / "portfolios" / u64 / "relationships" / "securities")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/portfolios/{pid}/relationships/securities".to_string(),
                    ))
                    .and(with_param("pid".to_string()))
                    .and_then(handlers::with_param),
            )
            .or(warp::path!(
                "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64
            )
            .and(filters.clone())
            .and(with_path_name(
                "/api/v1/portfolios/{pid}/securities/{sid}/transactions/{tid}".to_string(),
            ))
            .and(with_param("pid".to_string()))
            .and(with_param("sid".to_string()))
            .and(with_param("tid".to_string()))
            .and_then(handlers::with_3_params)),
    );

    let routes = get.or(post).or(patch).or(delete);

    let mut rx = listen_signals();

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async move {
            info!("waiting for signal");
            rx.recv().await;
            info!("shutdown signal received");
        });

    let result = tokio::try_join!(
        tokio::task::spawn(server),
        tokio::task::spawn(async move {
            let router = Arc::clone(&router);
            let mut router = router.write().await;

            router.run().await;
        }),
        tokio::task::spawn(async move {
            sub_svc.read().await.subscribe().await;
        }),
    );

    match result {
        Ok(_) => info!("run tasks"),
        Err(e) => error!("thread join error {}", e),
    }

    rmq.close();

    Ok(())
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

fn with_publisher(
    publisher: Arc<RwLock<Publisher>>,
) -> impl Filter<Extract = (Arc<RwLock<Publisher>>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&publisher))
}

fn with_router(
    r: Arc<RwLock<Router>>,
) -> impl Filter<Extract = (Arc<RwLock<Router>>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&r))
}

fn with_path_name(name: String) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    warp::any().map(move || name.clone())
}

fn with_param(name: String) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    warp::any().map(move || name.clone())
}
