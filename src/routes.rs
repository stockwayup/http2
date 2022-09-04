use std::sync::Arc;

use axum::http::header;
use axum::{
    http::Method,
    routing::{delete, get, post},
    Extension, Router,
};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::broker::Broker;
use crate::handlers::*;
use crate::publisher::Publisher;

const BODY_SIZE: u64 = 1024 * 250;

pub fn build_routes(pub_svc: Arc<RwLock<Publisher>>, broker: Arc<Broker>) -> Router {
    let router = Router::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH])
        .allow_headers([header::CONTENT_TYPE]);

    router
        .route("/api/v1/statuses", get(health_check))
        .route("/api/v1/users", post(proxy))
        .route("/api/v1/users/:uid/news", get(proxy))
        .route("/api/v1/users/{uid:[0-9]+}/news", get(proxy))
        .route("/api/v1/users/{uid:[0-9]+}/earnings", get(proxy))
        .route("/api/v1/users/{uid:[0-9]+}/dividends", get(proxy))
        .route("/api/v1/users/{uid:[0-9]+}", get(proxy))
        .route("/api/v1/users/{uid:[0-9]+}/day-prices", get(proxy))
        .route("/api/v1/users/{uid:[0-9]+}/day-price-periods", get(proxy))
        .route("/api/v1/users/{uid:[0-9]+}/view-history", get(proxy))
        .route("/api/v1/refresh-tokens", post(proxy))
        .route("/api/v1/refresh-tokens/{refresh-token}", delete(proxy))
        .route("/api/v1/sessions", post(proxy))
        .route("/api/v1/confirmation-codes", get(proxy))
        .route("/api/v1/confirmation-codes/{id}", post(proxy))
        .route("/api/v1/password-confirmation-codes", post(proxy))
        .route("/api/v1/password-confirmation-codes/{id}", post(proxy))
        .route("/api/v1/plans", get(proxy))
        .route("/api/v1/portfolios", get(proxy).post(proxy))
        .route(
            "/api/v1/portfolios/{pid:[0-9]+}",
            get(proxy).patch(proxy).delete(proxy),
        )
        .route(
            "/api/v1/portfolios/{pid:[0-9]+}/relationships/securities",
            post(proxy).delete(proxy),
        )
        .route(
            "/api/v1/portfolios/{pid:[0-9]+}/securities/{sid:[0-9]+}/transactions",
            get(proxy).post(proxy),
        )
        .route(
            "/api/v1/portfolios/{pid:[0-9]+}/securities/{sid:[0-9]+}/transactions/{tid:[0-9]+}",
            get(proxy).patch(proxy).delete(proxy),
        )
        .route("/api/v1/portfolios/{pid:[0-9]+}/securities", get(proxy))
        .route("/api/v1/portfolios/{pid:[0-9]+}/news", get(proxy))
        .route("/api/v1/portfolios/{pid:[0-9]+}/earnings", get(proxy))
        .route("/api/v1/portfolios/{pid:[0-9]+}/dividends", get(proxy))
        .route("/api/v1/portfolios/{pid:[0-9]+}/day-prices", get(proxy))
        .route(
            "/api/v1/portfolios/{pid:[0-9]+}/day-price-periods",
            get(proxy),
        )
        .route("/api/v1/securities", get(proxy))
        .route("/api/v1/securities/{sid:[0-9]+}/news", get(proxy))
        .route("/api/v1/securities/{sid:[0-9]+}/day-prices", get(proxy))
        .route(
            "/api/v1/securities/{sid:[0-9]+}/day-price-periods",
            get(proxy),
        )
        .route(
            "/api/v1/securities/{sid:[0-9]+}/quarterly-balance-sheet",
            get(proxy),
        )
        .route(
            "/api/v1/securities/{sid:[0-9]+}/annual-balance-sheet",
            get(proxy),
        )
        .route(
            "/api/v1/securities/{sid:[0-9]+}/quarterly-income-statements",
            get(proxy),
        )
        .route(
            "/api/v1/securities/{sid:[0-9]+}/annual-income-statements",
            get(proxy),
        )
        .route("/api/v1/securities/{sid:[0-9]+}", get(proxy))
        .route("/api/v1/countries", get(proxy))
        .route("/api/v1/currencies", get(proxy))
        .route("/api/v1/sectors", get(proxy))
        .route("/api/v1/industries", get(proxy))
        .route("/api/v1/exchanges", get(proxy))
        .layer(Extension(pub_svc.clone()))
        .layer(Extension(broker.clone()))
        .layer(cors)
        .fallback(get(not_found))
}
