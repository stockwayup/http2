use std::sync::Arc;

use axum::http::header;
use axum::{
    http::Method,
    routing::{delete, get, post},
    Extension, Router,
};
use tokio::sync::RwLock;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
};

use crate::broker::Broker;
use crate::handlers::*;
use crate::publisher::Publisher;

const BODY_SIZE: usize = 1024 * 250;

pub fn build_routes(pub_svc: Arc<RwLock<Publisher>>, broker: Arc<Broker>) -> Router {
    let router = Router::new();

    let cors = CorsLayer::new()
        .allow_origin(Any) // todo: set values
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    router
        .route("/api/v1/statuses", get(health_check))
        .route("/api/v1/users", post(proxy))
        .route("/api/v1/users/:uid/news", get(proxy))
        .route("/api/v1/users/:uid/earnings", get(proxy))
        .route("/api/v1/users/:uid/dividends", get(proxy))
        .route("/api/v1/users/:uid", get(proxy))
        .route("/api/v1/users/:uid/day-prices", get(proxy))
        .route("/api/v1/users/:uid/day-price-periods", get(proxy))
        .route("/api/v1/users/:uid/view-history", get(proxy))
        .route("/api/v1/refresh-tokens", post(proxy))
        .route("/api/v1/refresh-tokens/:refresh-token}", delete(proxy))
        .route("/api/v1/sessions", post(proxy))
        .route("/api/v1/confirmation-codes", get(proxy))
        .route("/api/v1/confirmation-codes/:id", post(proxy))
        .route("/api/v1/password-confirmation-codes", post(proxy))
        .route("/api/v1/password-confirmation-codes/:id", post(proxy))
        .route("/api/v1/plans", get(proxy))
        .route("/api/v1/portfolios", get(proxy).post(proxy))
        .route(
            "/api/v1/portfolios/:pid",
            get(proxy).patch(proxy).delete(proxy),
        )
        .route(
            "/api/v1/portfolios/:pid/relationships/securities",
            post(proxy).delete(proxy),
        )
        .route(
            "/api/v1/portfolios/:pid/securities/:sid/transactions",
            get(proxy).post(proxy),
        )
        .route(
            "/api/v1/portfolios/:pid/securities/:sid/transactions/:tid",
            get(proxy).patch(proxy).delete(proxy),
        )
        .route("/api/v1/portfolios/:pid/securities", get(proxy))
        .route("/api/v1/portfolios/:pid/news", get(proxy))
        .route("/api/v1/portfolios/:pid/earnings", get(proxy))
        .route("/api/v1/portfolios/:pid/dividends", get(proxy))
        .route("/api/v1/portfolios/:pid/day-prices", get(proxy))
        .route("/api/v1/portfolios/:pid/day-price-periods", get(proxy))
        .route("/api/v1/securities", get(proxy))
        .route("/api/v1/securities/:sid/news", get(proxy))
        .route("/api/v1/securities/:sid/day-prices", get(proxy))
        .route("/api/v1/securities/:sid/day-price-periods", get(proxy))
        .route(
            "/api/v1/securities/:sid/quarterly-balance-sheet",
            get(proxy),
        )
        .route("/api/v1/securities/:sid/annual-balance-sheet", get(proxy))
        .route(
            "/api/v1/securities/:sid/quarterly-income-statements",
            get(proxy),
        )
        .route(
            "/api/v1/securities/:sid/annual-income-statements",
            get(proxy),
        )
        .route("/api/v1/securities/:sid", get(proxy))
        .route("/api/v1/countries", get(proxy))
        .route("/api/v1/currencies", get(proxy))
        .route("/api/v1/sectors", get(proxy))
        .route("/api/v1/industries", get(proxy))
        .route("/api/v1/exchanges", get(proxy))
        .layer(Extension(pub_svc.clone()))
        .layer(Extension(broker.clone()))
        .layer(RequestBodyLimitLayer::new(BODY_SIZE))
        .layer(cors)
        .fallback(get(not_found))
}
