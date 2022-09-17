use std::collections::HashMap;

use axum::extract::MatchedPath;
use axum::http;
use serde::Serialize;
use serde_bytes_wrapper::Bytes;

#[derive(Serialize)]
pub struct HttpReq<'a> {
    pub r#type: String,
    pub access_token: String,
    pub method: String,
    pub user_values: HashMap<String, String>,
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
    pub val: HashMap<String, Bytes>,
}

impl<'a> HttpReq<'a> {
    pub fn new(
        uri: http::Uri,
        matched_path: MatchedPath,
        method: String,
        authorization: String,
        user_values: HashMap<String, String>,
        query_args: HashMap<String, String>,
        body: &'a [u8],
    ) -> HttpReq<'a> {
        let mut args = HashMap::new();

        for (key, val) in query_args {
            args.insert(key, val.as_bytes().to_vec().into());
        }

        HttpReq {
            r#type: matched_path.as_str().to_string(),
            access_token: authorization,
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
                args: Args { val: args },
            },
            body,
        }
    }
}
