use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{OriginalUri, Query};
use axum::headers::authorization::{Authorization, Bearer};
use axum::headers::HeaderValue;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, Response, StatusCode};
use axum::response::IntoResponse;
use axum::{body, body::Bytes, Extension, Json};
use kv_log_macro as log;
use serde::Serialize;
use serde_json::Serializer;
use tokio::sync::RwLock;
use tokio::time;

use crate::broker::Broker;
use crate::extractor::OptionalHeader;
use crate::publisher::Publisher;
use crate::responses::errors::{Error, Errors};

use super::events::HttpReq;
use super::responses::statuses::{Attributes, Statuses, StatusesData};

const TIMEOUT: u64 = 5;
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
    let errors = Errors {
        errors: vec![Error {
            code: "404".to_string(),
            title: "Not found".to_string(),
            detail: "The requested resource could not be found.".to_string(),
        }],
    };

    let mut buf = Vec::new();

    let mut se = Serializer::new(&mut buf);

    errors.serialize(&mut se).unwrap();

    let mut resp = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(http_body::Full::from(buf))
        .unwrap();

    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(JSON_API_TYPE));

    resp
}

pub async fn proxy(
    OriginalUri(uri): OriginalUri,
    method: Method,
    body: Bytes,
    Query(query_args): Query<HashMap<String, Vec<u8>>>,
    authorization: OptionalHeader<Authorization<Bearer>>,
    Extension(pub_svc): Extension<Arc<RwLock<Publisher>>>,
    Extension(broker): Extension<Arc<Broker>>,
) -> impl IntoResponse {
    let user_values: HashMap<String, String> = HashMap::new();

    let req = HttpReq::new(
        method.to_string(),
        uri,
        Option::Some(authorization.token().to_string()),
        user_values,
        query_args,
        &body,
    );

    let publ = pub_svc.read().await;

    let id = publ
        .publish(req)
        .await
        .map_err(|e| {
            log::error!("can't connect to rmq, {}", e);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                body::boxed(body::Empty::new()),
            )
                .into_response();
        })
        .unwrap();

    log::info!("request published", {id: id.clone().as_str()});

    let mut ch = broker.subscribe(id.clone());

    tokio::select! {
        event = ch.recv() => {
            broker.unsubscribe(id.clone());

            log::info!("response received", {id: id.clone().as_str()});

            let e = event.unwrap();

            let mut resp = Response::builder()
                .status(StatusCode::from_u16(e.code.parse::<u16>().unwrap()).unwrap())
                .body(http_body::Full::from(e.data))
                .unwrap();

            resp.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static(JSON_API_TYPE),
            );

            resp
        }
        _ = time::sleep(time::Duration::from_secs(TIMEOUT)) => {
            log::warn!("request timeout", {id: id.clone().as_str()});

            broker.unsubscribe(id.clone());

            let errors = Errors {
                errors: vec![
                    Error {
                        code: "408".to_string(),
                        title: "Request timeout".to_string(),
                        detail: "The server timed out waiting for the request.".to_string(),
                    }
                ]
            };

            let mut buf = Vec::new();

            let mut se = Serializer::new(&mut buf);

            errors.serialize(&mut se).unwrap();

            let mut resp = Response::builder()
                .status(StatusCode::REQUEST_TIMEOUT,)
                .body(http_body::Full::from(buf))
                .unwrap();

            resp.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static(JSON_API_TYPE),
            );

            resp
        }
    }
}
