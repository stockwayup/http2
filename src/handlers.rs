use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{MatchedPath, OriginalUri, Path, Query};
use axum::headers::{
    authorization::{Authorization, Bearer},
    HeaderValue,
};
use axum::http::{header::CONTENT_TYPE, Method, Response, StatusCode};
use axum::response::IntoResponse;
use axum::{body, body::Bytes, Extension, Json, TypedHeader};
use kv_log_macro as log;
use serde::Serialize;
use serde_json::Serializer;
use tokio::{sync::RwLock, time};

use crate::broker::Broker;
use crate::publisher::Publisher;
use crate::responses::errors::{Error, Errors};

use super::events::HttpReq;
use super::responses::statuses::{Attributes, Statuses, StatusesData};

const TIMEOUT: u64 = 30;
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
    matched_path: MatchedPath,
    method: Method,
    body: Bytes,
    Path(user_values): Path<HashMap<String, String>>,
    Query(query_args): Query<HashMap<String, String>>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    Extension(pub_svc): Extension<Arc<RwLock<Publisher>>>,
    Extension(broker): Extension<Arc<Broker>>,
) -> impl IntoResponse {
    let token: String = match authorization {
        None => "".to_string(),
        Some(val) => val.token().to_string(),
    };

    let req = HttpReq::new(
        uri,
        matched_path.clone(),
        method.to_string(),
        token,
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
        .unwrap(); // todo: check publish and repeat

    log::info!("request published", {id: id.clone().as_str(), path: matched_path.clone().as_str()});

    let mut ch = broker.subscribe(id.clone());

    tokio::select! {
        event = ch.recv() => {
            broker.unsubscribe(id.clone());

            let e = event.unwrap();

            log::info!("response received", {id: id.clone().as_str(), path: matched_path.clone().as_str(), code: e.code.as_str()});

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
            log::warn!("request timeout", {id: id.clone().as_str(), path: matched_path.clone().as_str()});

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
