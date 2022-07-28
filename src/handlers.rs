use std::collections::HashMap;
use std::convert::Infallible;
use std::result::Result as StdResult;
use std::sync::Arc;

use bytes::Buf;
use log::error;
use serde::Serialize;
use tokio::sync::Mutex;
use warp::{Rejection, Reply};
use warp::filters::route::Info;
use warp::http::{header::CONTENT_TYPE, HeaderValue};

use crate::publisher::Publisher;

use super::events::HttpReq;
use super::responses::{Attributes, Statuses, StatusesData};

type WebResult<T> = StdResult<T, Rejection>;

pub async fn health_check() -> Result<impl warp::Reply, Infallible> {
    let statuses = Statuses {
        data: StatusesData {
            id: "1".to_string(),
            r#type: "statuses".to_string(),
            attributes: Attributes {
                name: "success".to_string()
            },
        }
    };

    let mut resp = warp::reply::json(&statuses).into_response();

    resp.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/vnd.api+json"));

    Ok(resp)
}

pub async fn with_body<'a>(
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<Mutex<Publisher>>,
    body: bytes::Bytes,
    route_name: String,
) -> WebResult<impl Reply> {
    let user_values: HashMap<String, String> = HashMap::new();

    let b = &body.chunk();

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, b);

    let publ = publisher.lock().await;

    publ.publish(req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_body_and_param<'a, T>(
    param_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<Mutex<Publisher>>,
    body: bytes::Bytes,
    route_name: String,
    param_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param_name, param_value);

    let b = &body.chunk();

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, b);

    let publ = publisher.lock().await;

    publ.publish(req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_body_and_2_params<'a, T>(
    param1_value: T,
    param2_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<Mutex<Publisher>>,
    body: bytes::Bytes,
    route_name: String,
    param1_name: String,
    param2_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);

    let b = &body.chunk();

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, b);

    let publ = publisher.lock().await;

    publ.publish(req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_body_and_3_params<'a, T>(
    param1_value: T,
    param2_value: T,
    param3_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<Mutex<Publisher>>,
    body: bytes::Bytes,
    route_name: String,
    param1_name: String,
    param2_name: String,
    param3_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);
    user_values.insert(param3_name, param3_value);

    let b = &body.chunk();

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, b);

    let publ = publisher.lock().await;

    publ.publish(req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn handle<'a>(
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<Mutex<Publisher>>,
    route_name: String,
) -> WebResult<impl Reply> {
    let user_values: HashMap<String, String> = HashMap::new();

    let b: [u8; 0] = [];

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, &b);

    let publ = publisher.lock().await;

    publ.publish(req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_param<'a, T>(
    param_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<Mutex<Publisher>>,
    route_name: String,
    param_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param_name, param_value);

    let b: [u8; 0] = [];

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, &b);

    let publ = publisher.lock().await;

    publ.publish(req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_2_params<'a, T>(
    param1_value: T,
    param2_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<Mutex<Publisher>>,
    route_name: String,
    param1_name: String,
    param2_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);

    let b: [u8; 0] = [];

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, &b);

    let publ = publisher.lock().await;

    publ.publish(req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}


pub async fn with_3_params<'a, T>(
    param1_value: T,
    param2_value: T,
    param3_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    publisher: Arc<Mutex<Publisher>>,
    route_name: String,
    param1_name: String,
    param2_name: String,
    param3_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);
    user_values.insert(param3_name, param3_value);

    let b: [u8; 0] = [];

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, &b);

    let publ = publisher.lock().await;

    publ.publish(req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}
