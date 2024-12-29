use async_nats::Client;
use axum::http::{header, HeaderValue};
use axum::{
    http::Method,
    routing::{delete, get, post},
    Extension, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};

use crate::handlers::*;

const BODY_SIZE: usize = 1024 * 250;
const API_V1: &str = "/api/v1";

pub fn build_routes(
    allowed_origins: Vec<String>,
    nats: Arc<RwLock<Client>>,
) -> Router {
    let mut origins: Vec<HeaderValue> = Vec::new();

    for origin in allowed_origins {
        origins.push(HeaderValue::from_str(origin.as_str()).unwrap());
    }

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .route(&format!("{}/statuses", API_V1), get(health_check))
        .route(&format!("{}/users", API_V1), post(proxy))
        .route(&format!("{}/users/:uid/news", API_V1), get(proxy))
        .route(&format!("{}/users/:uid/earnings", API_V1), get(proxy))
        .route(&format!("{}/users/:uid/dividends", API_V1), get(proxy))
        .route(&format!("{}/users/:uid", API_V1), get(proxy))
        .route(&format!("{}/users/:uid/day-prices", API_V1), get(proxy))
        .route(&format!("{}/users/:uid/day-price-periods", API_V1), get(proxy))
        .route(&format!("{}/users/:uid/view-history", API_V1), get(proxy))
        .route(&format!("{}/refresh-tokens", API_V1), post(proxy))
        .route(&format!("{}/refresh-tokens/:refresh-token", API_V1), delete(proxy))
        .route(&format!("{}/sessions", API_V1), post(proxy))
        .route(&format!("{}/confirmation-codes", API_V1), get(proxy))
        .route(&format!("{}/confirmation-codes/:id", API_V1), post(proxy))
        .route(&format!("{}/password-confirmation-codes", API_V1), post(proxy))
        .route(&format!("{}/password-confirmation-codes/:id", API_V1), post(proxy))
        .route(&format!("{}/plans", API_V1), get(proxy))
        .route(&format!("{}/portfolios", API_V1), get(proxy).post(proxy))
        .route(&format!("{}/portfolios/:pid", API_V1), get(proxy).patch(proxy).delete(proxy))
        .route(&format!("{}/portfolios/:pid/relationships/securities", API_V1), post(proxy).delete(proxy))
        .route(&format!("{}/portfolios/:pid/securities/:sid/transactions", API_V1), get(proxy).post(proxy))
        .route(&format!("{}/portfolios/:pid/securities/:sid/transactions/:tid", API_V1), get(proxy).patch(proxy).delete(proxy))
        .route(&format!("{}/portfolios/:pid/securities", API_V1), get(proxy))
        .route(&format!("{}/portfolios/:pid/news", API_V1), get(proxy))
        .route(&format!("{}/portfolios/:pid/earnings", API_V1), get(proxy))
        .route(&format!("{}/portfolios/:pid/dividends", API_V1), get(proxy))
        .route(&format!("{}/portfolios/:pid/day-prices", API_V1), get(proxy))
        .route(&format!("{}/portfolios/:pid/day-price-periods", API_V1), get(proxy))
        .route(&format!("{}/securities", API_V1), get(proxy))
        .route(&format!("{}/securities/:sid/news", API_V1), get(proxy))
        .route(&format!("{}/securities/:sid/day-prices", API_V1), get(proxy))
        .route(&format!("{}/securities/:sid/day-price-periods", API_V1), get(proxy))
        .route(&format!("{}/securities/:sid/quarterly-balance-sheet", API_V1), get(proxy))
        .route(&format!("{}/securities/:sid/annual-balance-sheet", API_V1), get(proxy))
        .route(&format!("{}/securities/:sid/quarterly-income-statements", API_V1), get(proxy))
        .route(&format!("{}/securities/:sid/annual-income-statements", API_V1), get(proxy))
        .route(&format!("{}/securities/:sid", API_V1), get(proxy))
        .route(&format!("{}/countries", API_V1), get(proxy))
        .route(&format!("{}/currencies", API_V1), get(proxy))
        .route(&format!("{}/sectors", API_V1), get(proxy))
        .route(&format!("{}/industries", API_V1), get(proxy))
        .route(&format!("{}/exchanges", API_V1), get(proxy))
        .layer(Extension(nats))
        .layer(RequestBodyLimitLayer::new(BODY_SIZE))
        .layer(cors)
        .fallback(get(not_found))
}