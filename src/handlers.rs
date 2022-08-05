use std::collections::HashMap;
use std::convert::Infallible;
use std::result::Result as StdResult;
use std::sync::Arc;

use bytes::Buf;
use kv_log_macro as log;
use serde::Serialize;
use tokio::sync::RwLock;
use warp::filters::route::Info;
use warp::http::{header::CONTENT_TYPE, HeaderValue, StatusCode};
use warp::{Rejection, Reply};

use crate::publisher::Publisher;
use crate::router::Router;

use super::events::HttpReq;
use super::responses::{Attributes, Statuses, StatusesData};

type WebResult<T> = StdResult<T, Rejection>;

pub async fn health_check() -> Result<impl warp::Reply, Infallible> {
    let statuses = Statuses {
        data: StatusesData {
            id: "1".to_string(),
            r#type: "statuses".to_string(),
            attributes: Attributes {
                name: "success".to_string(),
            },
        },
    };

    let mut resp = warp::reply::json(&statuses).into_response();

    resp.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.api+json"),
    );

    Ok(resp)
}

pub async fn with_body<'a>(
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    body: bytes::Bytes,
    route_name: String,
) -> WebResult<impl Reply> {
    let user_values: HashMap<String, String> = HashMap::new();

    let b = &body.chunk();

    process(
        &route,
        authorization,
        query_args,
        publisher,
        router,
        route_name,
        user_values,
        b,
    )
    .await
}

pub async fn with_body_and_param<'a, T>(
    param_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    body: bytes::Bytes,
    route_name: String,
    param_name: String,
) -> WebResult<impl Reply>
where
    T: Serialize,
{
    let mut user_values = HashMap::new();

    user_values.insert(param_name, param_value);

    let b = &body.chunk();

    process(
        &route,
        authorization,
        query_args,
        publisher,
        router,
        route_name,
        user_values,
        b,
    )
    .await
}

pub async fn with_body_and_2_params<'a, T>(
    param1_value: T,
    param2_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    body: bytes::Bytes,
    route_name: String,
    param1_name: String,
    param2_name: String,
) -> WebResult<impl Reply>
where
    T: Serialize,
{
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);

    let b = &body.chunk();

    process(
        &route,
        authorization,
        query_args,
        publisher,
        router,
        route_name,
        user_values,
        b,
    )
    .await
}

pub async fn with_body_and_3_params<'a, T>(
    param1_value: T,
    param2_value: T,
    param3_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    body: bytes::Bytes,
    route_name: String,
    param1_name: String,
    param2_name: String,
    param3_name: String,
) -> WebResult<impl Reply>
where
    T: Serialize,
{
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);
    user_values.insert(param3_name, param3_value);

    let b = &body.chunk();

    process(
        &route,
        authorization,
        query_args,
        publisher,
        router,
        route_name,
        user_values,
        b,
    )
    .await
}

pub async fn handle<'a>(
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    route_name: String,
) -> WebResult<impl Reply> {
    let user_values: HashMap<String, String> = HashMap::new();

    let b: [u8; 0] = [];

    process(
        &route,
        authorization,
        query_args,
        publisher,
        router,
        route_name,
        user_values,
        &b,
    )
    .await
}

pub async fn with_param<'a, T>(
    param_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    route_name: String,
    param_name: String,
) -> WebResult<impl Reply>
where
    T: Serialize,
{
    let mut user_values = HashMap::new();

    user_values.insert(param_name, param_value);

    let b: [u8; 0] = [];

    process(
        &route,
        authorization,
        query_args,
        publisher,
        router,
        route_name,
        user_values,
        &b,
    )
    .await
}

pub async fn with_2_params<'a, T>(
    param1_value: T,
    param2_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    route_name: String,
    param1_name: String,
    param2_name: String,
) -> WebResult<impl Reply>
where
    T: Serialize,
{
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);

    let b: [u8; 0] = [];

    process(
        &route,
        authorization,
        query_args,
        publisher,
        router,
        route_name,
        user_values,
        &b,
    )
    .await
}

pub async fn with_3_params<'a, T>(
    param1_value: T,
    param2_value: T,
    param3_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    route_name: String,
    param1_name: String,
    param2_name: String,
    param3_name: String,
) -> WebResult<impl Reply>
where
    T: Serialize,
{
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);
    user_values.insert(param3_name, param3_value);

    let b: [u8; 0] = [];

    process(
        &route,
        authorization,
        query_args,
        publisher,
        router,
        route_name,
        user_values,
        &b,
    )
    .await
}

async fn process<T>(
    route: &Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
    route_name: String,
    user_values: HashMap<String, T>,
    body: &[u8],
) -> WebResult<impl Reply>
where
    T: Serialize,
{
    let mut qa: HashMap<String, Vec<u8>> = HashMap::new();

    for (key, value) in query_args {
        qa.insert(key, value.into_bytes());
    }

    let req = HttpReq::new(&route, route_name, authorization, user_values, qa, body);

    let publ = publisher.read().await;

    let id = publ.publish(req).await.map_err(|e| {
        log::error!(target: "app", "can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    log::info!(target: "app", "request published", {id: id.clone().as_str()});

    let mut ch = router.subscribe(id.clone());

    tokio::select! {
        event = ch.recv() => {
            router.unsubscribe(id.clone());

            log::info!(target: "app", "response received", {id: id.clone().as_str()});

            let e = event.unwrap();

            let mut resp = warp::reply::html(e.data).into_response();

            resp.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/vnd.api+json"),
            );

            *resp.status_mut() = StatusCode::from_u16(e.code.parse::<u16>().unwrap()).unwrap();

            Ok(resp)
        }
    }

    // todo: timeout
}
