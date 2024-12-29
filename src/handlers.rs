use crate::responses::errors::{Error, Errors};
use async_nats::{Client, HeaderMap};
use axum::extract::{MatchedPath, OriginalUri, Path, Query};
use axum::headers::{
    authorization::{Authorization, Bearer},
    HeaderValue,
};
use axum::http::{header::CONTENT_TYPE, Method, Response, StatusCode};
use axum::response::IntoResponse;
use axum::{body::Bytes, Extension, Json, TypedHeader};
use http_body::Full;
use kv_log_macro as log;
use serde::Serialize;
use rmp_serde::Serializer;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::time::Instant;

use super::events::HttpReq;
use super::responses::statuses::{Attributes, Statuses, StatusesData};

const SUBJECT: &str = "http";
const JSON_API_TYPE: &str = "application/vnd.api+json";

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

#[allow(clippy::too_many_arguments)]
pub async fn proxy(
    OriginalUri(uri): OriginalUri,
    matched_path: MatchedPath,
    method: Method,
    body: Bytes,
    Path(user_values): Path<HashMap<String, String>>,
    Query(query_args): Query<HashMap<String, String>>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    Extension(nats): Extension<Arc<RwLock<Client>>>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let id = Uuid::new_v4();

    log::info!("request received", {id: id.to_string().as_str(), path: matched_path.clone().as_str()});

    let req = HttpReq::new(
        uri,
        matched_path.clone(),
        method.to_string(),
        authorization.map_or_else(|| "".to_string(), |val| val.token().to_string()),
        user_values,
        query_args,
        &body,
    );

    let mut headers = HeaderMap::new();
    headers.insert("id", id.to_string());

    let client = nats.read().await;

    let mut buf = Vec::new();

    let mut se = Serializer::new(&mut buf).with_struct_map();

    req.serialize(&mut se).unwrap();

    let status_code: String;

    let resp = match client
        .request_with_headers(SUBJECT, headers, Bytes::from(buf))
        .await
    {
        Ok(response) => {
            let headers = response.headers.unwrap();
            let status_value = headers.get("code").cloned().unwrap_or_else(|| async_nats::HeaderValue::from("500"));

            status_code = StatusCode::from_bytes(status_value.to_string().as_bytes()).unwrap().to_string();
            create_response(StatusCode::from_bytes(status_value.to_string().as_bytes()).unwrap(), response.payload.to_vec()).into_response()
        }
        Err(e) => {
            status_code = StatusCode::REQUEST_TIMEOUT.to_string();
            log::error!("proxy request error: {}", e);
            create_error_response(StatusCode::REQUEST_TIMEOUT, "408", "Request timeout", "")
                .into_response()
        }
    };

    let elapsed_time = start_time.elapsed();

    log::info!("request processed", {
        id: id.to_string().as_str(),
        path: matched_path.clone().as_str(),
        code: status_code.as_str(),
        elapsed_time: format!("{:?}", elapsed_time).as_str(),
    });

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
    errors.serialize(&mut se).unwrap();

    let mut resp = Response::builder()
        .status(status)
        .body(http_body::Full::from(buf))
        .unwrap();

    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(JSON_API_TYPE));
    resp
}

fn create_response(status: StatusCode, data: Vec<u8>) -> Response<Full<axum::body::Bytes>> {
    let mut resp = Response::builder()
        .status(status)
        .body(http_body::Full::from(data))
        .unwrap();

    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(JSON_API_TYPE));
    resp
}
