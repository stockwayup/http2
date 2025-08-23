use crate::client_ip::extract_client_ip;
use crate::responses::errors::{Error, Errors};
use async_nats::HeaderMap;
use axum::extract::{MatchedPath, OriginalUri, Path, Query};
use axum::http::{header::CONTENT_TYPE, HeaderMap as HttpHeaderMap, Method, Response, StatusCode};
use axum::response::IntoResponse;
use axum::{body::Bytes, extract::State, Json};
use http::{header::AUTHORIZATION, HeaderValue};
use http_body_util::Full;
use rmp_serde::Serializer;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, error, info, instrument, Span};
use uuid::Uuid;

use super::events::{HttpReq, HttpRequestInfo, RequestContext};
use super::responses::statuses::{Attributes, Statuses, StatusesData};
use crate::types::{AuthToken, HttpMethod};

const SUBJECT: &str = "http";
const JSON_API_TYPE: &str = "application/vnd.api+json";

// Simplified header management for future OpenTelemetry integration

pub async fn health_check() -> impl IntoResponse {
    let statuses = Statuses {
        data: StatusesData {
            id: "1".to_string(),
            r#type: "statuses".to_string(),
            attributes: Attributes {
                name: "success".to_string(),
            },
        },
    };

    let mut resp = (StatusCode::OK, Json(statuses)).into_response();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(JSON_API_TYPE));
    resp
}

pub async fn not_found() -> impl IntoResponse {
    create_error_response(
        StatusCode::NOT_FOUND,
        "404",
        "Not found",
        "The requested resource could not be found.",
    )
}

pub async fn metrics_handler(
    State(shared_state): State<crate::types::SharedState>,
) -> impl IntoResponse {
    let metrics = &shared_state.metrics;
    match metrics {
        Some(metrics) => match metrics.render().await {
            Ok(body) => match Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
                .body(body)
            {
                Ok(response) => response.into_response(),
                Err(e) => {
                    error!(error = %e, "failed to build metrics response");
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Internal server error".to_string())
                        .unwrap_or_else(|_| Response::new("Fatal error".to_string()))
                        .into_response()
                }
            },
            Err(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error rendering metrics".to_string())
                .unwrap_or_else(|_| Response::new("Error rendering metrics".to_string()))
                .into_response(),
        },
        None => Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body("Metrics not available".to_string())
            .unwrap_or_else(|_| Response::new("Metrics not available".to_string()))
            .into_response(),
    }
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(body), fields(
    http.method = %method,
    http.route = %matched_path.as_str(),
    http.request.body.size = body.len(),
))]
pub async fn proxy(
    OriginalUri(uri): OriginalUri,
    matched_path: MatchedPath,
    method: Method,
    headers: HttpHeaderMap,
    Path(user_values): Path<HashMap<String, String>>,
    Query(query_args): Query<HashMap<String, String>>,
    State(shared_state): State<crate::types::SharedState>,
    body: Bytes,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let id = Uuid::new_v4();

    // Extract client IP from headers
    let client_ip = extract_client_ip(&headers);
    debug!(client_ip = client_ip.as_str(), "extracted client IP");

    // Add span attributes
    let span = Span::current();
    span.record("request.id", id.to_string().as_str());
    let authorization_header = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    span.record("user.authenticated", authorization_header.is_some());
    span.record("client.ip", client_ip.as_str());

    let http_info = HttpRequestInfo {
        uri,
        method: HttpMethod::from_axum_method(&method),
        matched_path: matched_path.clone(),
    };

    let context = RequestContext {
        client_ip,
        authorization: authorization_header
            .map_or_else(AuthToken::empty, |token| AuthToken::new(token.to_string())),
        user_values,
    };

    let req = HttpReq::new(http_info, context, query_args, &body);

    let mut headers = HeaderMap::new();
    headers.insert("id", id.to_string());

    // TODO: Add OpenTelemetry context propagation here

    let client = shared_state.nats.read().await;

    let mut buf = Vec::new();

    let mut se = Serializer::new(&mut buf).with_struct_map();

    if let Err(e) = req.serialize(&mut se) {
        error!(error = %e, request_id = %id, "failed to serialize request");
        return create_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "500",
            "Internal server error",
            "Failed to serialize request",
        )
        .into_response();
    }

    let status_code: String;

    let resp = match client
        .request_with_headers(SUBJECT, headers, Bytes::from(buf))
        .await
    {
        Ok(response) => {
            let headers = match response.headers {
                Some(headers) => headers,
                None => {
                    error!(request_id = %id, "NATS response missing headers");
                    return create_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "500",
                        "Internal server error",
                        "Invalid NATS response format",
                    )
                    .into_response();
                }
            };

            let status_value = headers
                .get("code")
                .cloned()
                .unwrap_or_else(|| async_nats::HeaderValue::from("500"));
            let code = match StatusCode::from_bytes(status_value.to_string().as_bytes()) {
                Ok(code) => code,
                Err(e) => {
                    error!(error = %e, request_id = %id, "invalid status code from NATS");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };

            status_code = code.to_string();
            span.record("http.response.status_code", code.as_u16() as i64);
            span.record("nats.response.size", response.payload.len() as i64);

            info!(status = code.as_u16(), "NATS response received");

            create_response(code, response.payload.to_vec()).into_response()
        }
        Err(e) => {
            status_code = StatusCode::REQUEST_TIMEOUT.to_string();
            span.record("http.response.status_code", 408_i64);
            span.record("error", true);

            error!(error = %e, "NATS request failed");

            create_error_response(StatusCode::REQUEST_TIMEOUT, "408", "Request timeout", "")
                .into_response()
        }
    };

    let elapsed_time = start_time.elapsed();

    // Record final span attributes
    span.record("duration_ms", elapsed_time.as_millis() as i64);

    // Record metrics if available
    if let Some(metrics) = &shared_state.metrics {
        let route_template = matched_path.as_str();
        let status_num: u16 = status_code.parse().unwrap_or(500);

        metrics.record_http_request(method.as_str(), route_template, status_num, elapsed_time);

        metrics.record_nats_request(SUBJECT, status_num < 400, elapsed_time);
    }

    info!(
        request_id = %id,
        method = %method,
        route = %matched_path.as_str(),
        status = status_code.as_str(),
        elapsed_time_ms = elapsed_time.as_millis(),
        "request completed"
    );

    resp
}

fn create_error_response(
    status: StatusCode,
    code: &str,
    title: &str,
    detail: &str,
) -> Response<Full<axum::body::Bytes>> {
    let errors = Errors {
        errors: vec![Error {
            code: code.to_string(),
            title: title.to_string(),
            detail: detail.to_string(),
        }],
    };

    let mut buf = Vec::new();
    let mut se = Serializer::new(&mut buf);

    // Handle serialization failure gracefully
    if let Err(e) = errors.serialize(&mut se) {
        error!(error = %e, "failed to serialize error response");
        // Return a minimal fallback response as JSON string
        let fallback_body = format!(
            r#"{{"errors":[{{"code":"{}","title":"{}","detail":"Serialization failed"}}]}}"#,
            code, title
        );
        buf = fallback_body.into_bytes();
    }

    match Response::builder().status(status).body(Full::from(buf)) {
        Ok(mut resp) => {
            resp.headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static(JSON_API_TYPE));
            resp
        }
        Err(e) => {
            error!(error = %e, "failed to build error response");
            // Return minimal fallback response - this should never fail
            Response::new(Full::from("Internal server error".as_bytes()))
        }
    }
}

fn create_response(status: StatusCode, data: Vec<u8>) -> Response<Full<axum::body::Bytes>> {
    match Response::builder().status(status).body(Full::from(data)) {
        Ok(mut resp) => {
            resp.headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static(JSON_API_TYPE));
            resp
        }
        Err(e) => {
            error!(error = %e, "failed to build response");
            // Return a minimal error response
            match Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::from("Failed to build response".as_bytes()))
            {
                Ok(resp) => resp,
                Err(_) => {
                    // Last resort fallback - this should never fail
                    Response::new(Full::from("Internal server error".as_bytes()))
                }
            }
        }
    }
}
