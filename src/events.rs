use std::collections::HashMap;

use axum::http;
use serde::Serialize;

const BEARER: &str = "Bearer ";

#[derive(Serialize)]
pub struct HttpReq<'a, T>
where
    T: Serialize,
{
    pub r#type: String,
    pub access_token: String,
    pub method: String,
    pub user_values: HashMap<String, T>,
    pub uri: Uri<'a>,
    #[serde(with = "serde_bytes")]
    pub body: &'a [u8],
}

#[derive(Serialize)]
pub struct Uri<'a> {
    #[serde(with = "serde_bytes")]
    pub path_original: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub scheme: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub path: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub query_string: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub hash: &'a [u8],
    #[serde(with = "serde_bytes")]
    pub host: Vec<u8>,
    pub args: Args,
}

#[derive(Serialize)]
pub struct Args {
    pub val: HashMap<String, Vec<u8>>,
}

impl<'a, T> HttpReq<'a, T>
where
    T: Serialize,
{
    pub fn new(
        method: String,
        uri: http::Uri,
        authorization: Option<String>,
        user_values: HashMap<String, T>,
        query_args: HashMap<String, Vec<u8>>,
        body: &'a [u8],
    ) -> HttpReq<'a, T> {
        HttpReq {
            r#type: "".to_string(),
            access_token: match authorization {
                None => "".to_string(),
                Some(t) => {
                    if t.contains(BEARER) {
                        let split = t.split_once(BEARER).unwrap();

                        split.1.to_string()
                    } else {
                        "".to_string()
                    }
                }
            },
            method: method.to_string(),
            user_values,
            uri: Uri {
                path_original: uri.to_string().into_bytes(),
                scheme: match uri.scheme() {
                    None => "".to_string().into_bytes(),
                    Some(s) => s.as_str().to_string().into_bytes(),
                },
                path: uri.path().to_string().into_bytes(),
                query_string: match uri.path_and_query().unwrap().query() {
                    None => "".to_string().into_bytes(),
                    Some(q) => q.to_string().into_bytes(),
                },
                hash: &[],
                host: match uri.host() {
                    None => "".to_string().into_bytes(),
                    Some(h) => h.to_string().into_bytes(),
                },
                args: Args { val: query_args },
            },
            body,
        }
    }
}
