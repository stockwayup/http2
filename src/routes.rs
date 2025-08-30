use axum::http::{header, HeaderValue};
use axum::{
    http::Method,
    routing::{any, delete, get, post},
    Router,
};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{debug, info_span, Span};

use crate::handlers::*;
use crate::types::SharedState;

const BODY_SIZE: usize = 1024 * 250;
const API_V1: &str = "/api/v1";

pub fn build_routes(
    allowed_origins: Vec<String>,
    enable_cors: bool,
    shared_state: SharedState,
) -> Router {
    let cors_layer = if enable_cors {
        let mut origins: Vec<HeaderValue> = Vec::new();

        for origin in allowed_origins {
            match HeaderValue::from_str(origin.as_str()) {
                Ok(header_value) => origins.push(header_value),
                Err(e) => {
                    tracing::error!(error = %e, origin = %origin, "invalid CORS origin header value, skipping");
                    continue;
                }
            }
        }

        Some(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
    } else {
        None
    };

    let mut router = Router::new().route(&format!("{}/statuses", API_V1), get(health_check));

    // Add /metrics endpoint if metrics are available
    if shared_state.metrics.is_some() {
        router = router.route("/metrics", get(metrics_handler));
    }

    let router = router
        .route(&format!("{}/users", API_V1), post(proxy))
        .route(&format!("{}/users/{{uid}}/news", API_V1), get(proxy))
        .route(&format!("{}/users/{{uid}}/earnings", API_V1), get(proxy))
        .route(&format!("{}/users/{{uid}}/dividends", API_V1), get(proxy))
        .route(&format!("{}/users/{{uid}}", API_V1), get(proxy))
        .route(&format!("{}/users/{{uid}}/day-prices", API_V1), get(proxy))
        .route(
            &format!("{}/users/{{uid}}/day-price-periods", API_V1),
            get(proxy),
        )
        .route(&format!("{}/users/{{uid}}/view-history", API_V1), get(proxy))
        .route(&format!("{}/refresh-tokens", API_V1), post(proxy))
        .route(
            &format!("{}/refresh-tokens/{{refresh-token}}", API_V1),
            delete(proxy),
        )
        .route(&format!("{}/sessions", API_V1), post(proxy))
        .route(&format!("{}/confirmation-codes", API_V1), get(proxy))
        .route(&format!("{}/confirmation-codes/{{id}}", API_V1), post(proxy))
        .route(
            &format!("{}/password-confirmation-codes", API_V1),
            post(proxy),
        )
        .route(
            &format!("{}/password-confirmation-codes/{{id}}", API_V1),
            post(proxy),
        )
        .route(&format!("{}/plans", API_V1), get(proxy))
        .route(&format!("{}/portfolios", API_V1), get(proxy).post(proxy))
        .route(
            &format!("{}/portfolios/{{pid}}", API_V1),
            get(proxy).patch(proxy).delete(proxy),
        )
        .route(
            &format!("{}/portfolios/{{pid}}/relationships/securities", API_V1),
            post(proxy).delete(proxy),
        )
        .route(
            &format!("{}/portfolios/{{pid}}/securities/{{sid}}/transactions", API_V1),
            get(proxy).post(proxy),
        )
        .route(
            &format!(
                "{}/portfolios/{{pid}}/securities/{{sid}}/transactions/{{tid}}",
                API_V1
            ),
            get(proxy).patch(proxy).delete(proxy),
        )
        .route(
            &format!("{}/portfolios/{{pid}}/securities", API_V1),
            get(proxy),
        )
        .route(&format!("{}/portfolios/{{pid}}/news", API_V1), get(proxy))
        .route(&format!("{}/portfolios/{{pid}}/earnings", API_V1), get(proxy))
        .route(&format!("{}/portfolios/{{pid}}/dividends", API_V1), get(proxy))
        .route(
            &format!("{}/portfolios/{{pid}}/day-prices", API_V1),
            get(proxy),
        )
        .route(
            &format!("{}/portfolios/{{pid}}/day-price-periods", API_V1),
            get(proxy),
        )
        .route(&format!("{}/securities", API_V1), get(proxy))
        .route(&format!("{}/securities/{{sid}}/news", API_V1), get(proxy))
        .route(
            &format!("{}/securities/{{sid}}/day-prices", API_V1),
            get(proxy),
        )
        .route(
            &format!("{}/securities/{{sid}}/day-price-periods", API_V1),
            get(proxy),
        )
        .route(
            &format!("{}/securities/{{sid}}/quarterly-balance-sheet", API_V1),
            get(proxy),
        )
        .route(
            &format!("{}/securities/{{sid}}/annual-balance-sheet", API_V1),
            get(proxy),
        )
        .route(
            &format!("{}/securities/{{sid}}/quarterly-income-statements", API_V1),
            get(proxy),
        )
        .route(
            &format!("{}/securities/{{sid}}/annual-income-statements", API_V1),
            get(proxy),
        )
        .route(&format!("{}/securities/{{sid}}", API_V1), get(proxy))
        .route(&format!("{}/countries", API_V1), get(proxy))
        .route(&format!("{}/currencies", API_V1), get(proxy))
        .route(&format!("{}/sectors", API_V1), get(proxy))
        .route(&format!("{}/industries", API_V1), get(proxy))
        .route(&format!("{}/exchanges", API_V1), get(proxy))
        .with_state(shared_state)
        .layer(RequestBodyLimitLayer::new(BODY_SIZE));

    let router = if let Some(cors) = cors_layer {
        router.layer(cors)
    } else {
        router
    };

    // Custom classifier that never treats HTTP status codes as failures
    // Only real connection/protocol errors will be classified as failures
    // Using range 600..=999 means no valid HTTP status codes will be treated as failures
    
    router
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<axum::extract::MatchedPath>()
                        .map(|mp| mp.as_str())
                        .unwrap_or("unknown");

                    info_span!(
                        "http_request",
                        method = %request.method(),
                        route = matched_path,
                        version = ?request.version(),
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>, latency: std::time::Duration, _span: &Span| {
                        let status = response.status();
                        let status_code = status.as_u16();
                        let latency_ms = latency.as_millis();
                        
                        // All responses logged at DEBUG level for detailed tracing
                        // The main log in handlers.rs provides the production-level logging
                        debug!(
                            status = status_code,
                            latency = ?latency_ms,
                            "tower_http trace"
                        );
                    },
                )
                .on_failure(
                    |error: tower_http::classify::ServerErrorsFailureClass,
                     latency: std::time::Duration,
                     _span: &Span| {
                        // This is now only called for real failures (not HTTP status codes)
                        // Such as connection errors, timeouts at TCP level, etc.
                        debug!(
                            error = ?error,
                            latency = ?latency.as_millis(),
                            "tower_http failure trace"
                        );
                    },
                )
                .on_request(|_request: &axum::http::Request<_>, _span: &Span| {
                    // Request start logged at DEBUG level
                    debug!("tower_http request start");
                })
                .on_body_chunk(|_chunk: &axum::body::Bytes, _latency: std::time::Duration, _span: &Span| {
                    // Body chunks logged at TRACE level (won't show even in debug mode)
                    tracing::trace!("tower_http body chunk");
                })
                .on_eos(|_trailers: Option<&http::HeaderMap>, _stream_duration: std::time::Duration, _span: &Span| {
                    // End of stream logged at TRACE level
                    tracing::trace!("tower_http stream end");
                }),
        )
        .fallback(any(not_found))
}
