use std::collections::HashMap;
use std::convert::Infallible;
use std::ops::Add;
use std::result::Result as StdResult;
use std::time::Duration;

use bytes::Buf;
use deadpool_lapin::{Pool, PoolError};
use lapin::options::BasicPublishOptions;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::{ShortShortUInt, ShortString};
use log::error;
use rmp_serde::Serializer;
use serde::Serialize;
use uuid::Uuid;
use warp::{Rejection, Reply};
use warp::filters::route::Info;
use warp::http::{header::CONTENT_TYPE, HeaderValue};

use super::events::HttpReq;
use super::responses::{Attributes, Statuses, StatusesData};

type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;
type WebResult<T> = StdResult<T, Rejection>;
type RMQResult<T> = StdResult<T, PoolError>;

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

pub async fn with_body(
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    pool: Pool,
    body: bytes::Bytes,
    route_name: String,
) -> WebResult<impl Reply> {
    let user_values: HashMap<String, String> = HashMap::new();

    let b = &body.chunk();

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, b);

    publish(pool, req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_body_and_param<T>(
    param_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    pool: Pool,
    body: bytes::Bytes,
    route_name: String,
    param_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param_name, param_value);

    let b = &body.chunk();

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, b);

    publish(pool, req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_body_and_2_params<T>(
    param1_value: T,
    param2_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    pool: Pool,
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

    publish(pool, req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_body_and_3_params<T>(
    param1_value: T,
    param2_value: T,
    param3_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    pool: Pool,
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

    publish(pool, req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn handle(
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    pool: Pool,
    route_name: String,
) -> WebResult<impl Reply> {
    let user_values: HashMap<String, String> = HashMap::new();

    let b: [u8;0] = [];

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, &b);

    publish(pool, req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_param<T>(
    param_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    pool: Pool,
    route_name: String,
    param_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param_name, param_value);

    let b: [u8;0] = [];

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, &b);

    publish(pool, req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

pub async fn with_2_params<T>(
    param1_value: T,
    param2_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    pool: Pool,
    route_name: String,
    param1_name: String,
    param2_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);

    let b: [u8;0] = [];

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, &b);

    publish(pool, req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}


pub async fn with_3_params<T>(
    param1_value: T,
    param2_value: T,
    param3_value: T,
    route: Info,
    authorization: Option<String>,
    query_args: HashMap<String, String>,
    pool: Pool,
    route_name: String,
    param1_name: String,
    param2_name: String,
    param3_name: String,
) -> WebResult<impl Reply> where T: Serialize {
    let mut user_values = HashMap::new();

    user_values.insert(param1_name, param1_value);
    user_values.insert(param2_name, param2_value);
    user_values.insert(param3_name, param3_value);

    let b: [u8;0] = [];

    let req = HttpReq::new(&route, route_name, authorization, user_values, query_args, &b);

    publish(pool, req).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        warp::reject::reject()
    })?;

    Ok(warp::reply::html("here will be an answer"))
}

async fn get_rmq_con(pool: Pool) -> RMQResult<Connection> {
    let connection = pool.get().await?;

    Ok(connection)
}

async fn publish<'a, T: serde::Serialize>(pool: Pool, req: HttpReq<'a, T>) -> Result<(), Box<dyn std::error::Error>> {
    let rmq_con = get_rmq_con(pool).await.map_err(|e| {
        error!("can't connect to rmq, {}", e);

        e
    })?;

    let channel = rmq_con.create_channel().await.map_err(|e| {
        error!("can't create channel, {}", e);

        e
    })?;

    let mut buf = Vec::new();

    let mut se = Serializer::new(&mut buf)
        .with_struct_map();

    req.serialize(&mut se).unwrap();

    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let expiration = since_the_epoch.add(Duration::from_secs(120)).as_secs().to_string();

    let props = AMQPProperties::default().
        with_content_type(ShortString::from("application/octet-stream")).
        with_message_id(ShortString::from(Uuid::new_v4().to_string())).
        with_delivery_mode(ShortShortUInt::from(1)).
        with_expiration(ShortString::from(expiration))
        ;

    channel
        .basic_publish(
            "",
            "http.requests",
            BasicPublishOptions::default(),
            buf.as_slice(),
            props,
        )
        .await
        .map_err(|e| {
            error!("can't publish: {}", e);

            e
        })?;

    Ok(())
}