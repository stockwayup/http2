use std::collections::HashMap;

use serde::Serialize;
use warp::filters::route::Info;

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
    pub body: &'a [u8],
}

#[derive(Serialize)]
pub struct Uri<'a> {
    pub path_original: &'a [u8],
    pub scheme: &'a [u8],
    pub path: &'a [u8],
    pub query_string: &'a [u8],
    pub hash: &'a [u8],
    pub host: &'a [u8],
    pub args: Args,
}

#[derive(Serialize)]
pub struct Args {
    pub val: HashMap<String, String>,
}

impl<'a, T> HttpReq<'a, T>
where
    T: Serialize,
{
    pub fn new(
        route: &'a Info,
        route_name: String,
        authorization: Option<String>,
        user_values: HashMap<String, T>,
        query_args: HashMap<String, String>,
        body: &'a [u8],
    ) -> HttpReq<'a, T> {
        HttpReq {
            r#type: route_name,
            access_token: match authorization {
                None => "".to_string(),
                Some(t) => {
                    if t.contains("Bearer ") {
                        let split = t.split_once("Bearer ").unwrap();

                        split.1.to_string()
                    } else {
                        "".to_string()
                    }
                }
            },
            method: route.method().to_string(),
            user_values,
            uri: Uri {
                path_original: route.uri().path_and_query().unwrap().as_str().as_bytes(),
                scheme: match route.uri().scheme_str() {
                    None => &[],
                    Some(s) => s.as_bytes(),
                },
                path: route.uri().path().as_bytes(),
                query_string: match route.uri().query() {
                    None => &[],
                    Some(qs) => qs.as_bytes(),
                },
                hash: &[],
                host: match route.host() {
                    None => &[],
                    Some(h) => h.as_bytes(),
                },
                args: Args { val: query_args },
            },
            body,
        }
    }
}
