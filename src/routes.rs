use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::RwLock;
use warp::filters::BoxedFilter;
use warp::{route, Filter, Reply};

use crate::handlers::*;
use crate::publisher::Publisher;
use crate::router::Router;

const BODY_SIZE: u64 = 1024 * 250;

pub fn build_routes(
    pub_svc: Arc<RwLock<Publisher>>,
    router: Arc<Router>,
) -> BoxedFilter<(impl Reply,)> {
    let filters = warp::any()
        .and(route())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::query::<HashMap<String, String>>())
        .and(with_publisher(pub_svc.clone()))
        .and(with_router(router.clone()));

    let body_filters = filters.clone().and(warp::body::bytes());

    let get = warp::get()
        .and(
            warp::path!("api" / "v1" / "statuses")
                .and_then(health_check)
                .or(warp::path!("api" / "v1" / "users" / u64 / "news")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/users/{uid}/news".to_string()))
                    .and(with_param_name("uid".to_string()))
                    .and_then(with_param))
                .or(warp::path!("api" / "v1" / "users" / u64 / "earnings")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/users/{uid}/earning".to_string()))
                    .and(with_param_name("uid".to_string()))
                    .and_then(with_param))
                .or(warp::path!("api" / "v1" / "users" / u64 / "dividends")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/users/{uid}/dividends".to_string()))
                    .and(with_param_name("uid".to_string()))
                    .and_then(with_param))
                .or(warp::path!("api" / "v1" / "users" / u64 / "day-prices")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/users/{uid}/day-prices".to_string()))
                    .and(with_param_name("uid".to_string()))
                    .and_then(with_param))
                .or(
                    warp::path!("api" / "v1" / "users" / u64 / "day-price-periods")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/users/{uid}/day-prices-periods".to_string(),
                        ))
                        .and(with_param_name("uid".to_string()))
                        .and_then(with_param),
                )
                .or(warp::path!("api" / "v1" / "users" / u64 / "view-history")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/users/{uid}/view-history".to_string(),
                    ))
                    .and(with_param_name("uid".to_string()))
                    .and_then(with_param))
                .or(warp::path!("api" / "v1" / "users" / u64)
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/users/{uid}".to_string()))
                    .and(with_param_name("uid".to_string()))
                    .and_then(with_param))
                .or(warp::path!("api" / "v1" / "confirmation-codes")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/confirmation-codes".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "plans")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/plans".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "countries")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/countries".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "currencies")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/currencies".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "sectors")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/sectors".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "industries")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/industries".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "exchanges")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/exchanges".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "portfolios")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/portfolios".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "portfolios" / u64)
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/portfolios/{pid}".to_string()))
                    .and(with_param_name("pid".to_string()))
                    .and_then(with_param))
                .or(warp::path!(
                    "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions"
                )
                .and(filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string(),
                ))
                .and(with_param_name("pid".to_string()))
                .and(with_param_name("sid".to_string()))
                .and_then(with_2_params))
                .or(warp::path!(
                    "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64
                )
                .and(filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string(),
                ))
                .and(with_param_name("pid".to_string()))
                .and(with_param_name("sid".to_string()))
                .and(with_param_name("tid".to_string()))
                .and_then(with_3_params))
                .or(
                    warp::path!("api" / "v1" / "portfolios" / u64 / "securities")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/portfolios/{pid}/securities".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(warp::path!("api" / "v1" / "portfolios" / u64 / "news")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/portfolios/{pid}/news".to_string()))
                    .and(with_param_name("pid".to_string()))
                    .and_then(with_param))
                .or(warp::path!("api" / "v1" / "portfolios" / u64 / "earnings")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/portfolios/{pid}/earnings".to_string(),
                    ))
                    .and(with_param_name("pid".to_string()))
                    .and_then(with_param))
                .or(warp::path!("api" / "v1" / "portfolios" / u64 / "dividends")
                    .and(filters.clone())
                    .and(with_path_name(
                        "/api/v1/portfolios/{pid}/dividends".to_string(),
                    ))
                    .and(with_param_name("pid".to_string()))
                    .and_then(with_param))
                .or(
                    warp::path!("api" / "v1" / "portfolios" / u64 / "day-prices")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/portfolios/{pid}/day-prices".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(
                    warp::path!("api" / "v1" / "portfolios" / u64 / "day-price-periods")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/portfolios/{pid}/day-price-periods".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(warp::path!("api" / "v1" / "securities")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/securities".to_string()))
                    .and_then(handle))
                .or(warp::path!("api" / "v1" / "securities" / u64 / "news")
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/securities/{sid}/news".to_string()))
                    .and(with_param_name("pid".to_string()))
                    .and_then(with_param))
                .or(
                    warp::path!("api" / "v1" / "securities" / u64 / "day-prices")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/securities/{sid}/day-prices".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(
                    warp::path!("api" / "v1" / "securities" / u64 / "day-price-periods")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/securities/{sid}/day-price-periods".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(
                    warp::path!("api" / "v1" / "securities" / u64 / "quarterly-balance-sheet")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/securities/{sid}/quarterly-balance-sheet".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(
                    warp::path!("api" / "v1" / "securities" / u64 / "annual-balance-sheet")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/securities/{sid}/quarterly-balance-sheet".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(
                    warp::path!("api" / "v1" / "securities" / u64 / "quarterly-income-statements")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/securities/{sid}/quarterly-income-statements".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(
                    warp::path!("api" / "v1" / "securities" / u64 / "annual-income-statements")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/securities/{sid}/annual-income-statements".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(warp::path!("api" / "v1" / "securities" / u64)
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/securities/{sid}".to_string()))
                    .and(with_param_name("pid".to_string()))
                    .and_then(with_param)),
        )
        .boxed();

    let post = warp::post()
        .and(warp::body::content_length_limit(BODY_SIZE))
        .and(
            warp::path!("api" / "v1" / "users")
                .and(body_filters.clone())
                .and(with_path_name("/api/v1/users".to_string()))
                .and_then(with_body)
                .or(warp::path!("api" / "v1" / "refresh-tokens")
                    .and(body_filters.clone())
                    .and(with_path_name("/api/v1/refresh-tokens".to_string()))
                    .and_then(with_body))
                .or(warp::path!("api" / "v1" / "sessions")
                    .and(body_filters.clone())
                    .and(with_path_name("/api/v1/sessions".to_string()))
                    .and_then(with_body))
                .or(warp::path!("api" / "v1" / "confirmation-codes" / String)
                    .and(body_filters.clone())
                    .and(with_path_name(
                        "/api/v1/confirmation-codes/{id}".to_string(),
                    ))
                    .and(with_param_name("id".to_string()))
                    .and_then(with_body_and_param))
                .or(warp::path!("api" / "v1" / "password-confirmation-codes")
                    .and(body_filters.clone())
                    .and(with_path_name(
                        "/api/v1/password-confirmation-codes".to_string(),
                    ))
                    .and_then(with_body))
                .or(warp::path!("api" / "v1" / "portfolios")
                    .and(body_filters.clone())
                    .and(with_path_name("/api/v1/portfolios".to_string()))
                    .and_then(with_body))
                .or(
                    warp::path!("api" / "v1" / "portfolios" / u64 / "relationships" / "securities")
                        .and(body_filters.clone())
                        .and(with_path_name(
                            "/api/v1/portfolios/{pid}/relationships/securities".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_body_and_param),
                )
                .or(warp::path!(
                    "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions"
                )
                .and(body_filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string(),
                ))
                .and(with_param_name("pid".to_string()))
                .and(with_param_name("sid".to_string()))
                .and_then(with_body_and_2_params)),
        )
        .boxed();

    let patch = warp::patch()
        .and(warp::body::content_length_limit(BODY_SIZE))
        .and(
            warp::path!("api" / "v1" / "portfolios" / u64)
                .and(body_filters.clone())
                .and(with_path_name("/api/v1/portfolios/{pid}".to_string()))
                .and(with_param_name("pid".to_string()))
                .and_then(with_body_and_param)
                .or(warp::path!(
                    "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64
                )
                .and(body_filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/securities/{sid}/transactions/{tid}".to_string(),
                ))
                .and(with_param_name("pid".to_string()))
                .and(with_param_name("sid".to_string()))
                .and(with_param_name("tid".to_string()))
                .and_then(with_body_and_3_params)),
        )
        .boxed();

    let delete = warp::delete()
        .and(
            warp::path!("api" / "v1" / "refresh-tokens" / u64)
                .and(filters.clone())
                .and(with_path_name(
                    "/api/v1/refresh-tokens/{refresh-token}".to_string(),
                ))
                .and(with_param_name("refresh-token".to_string()))
                .and_then(with_param)
                .or(warp::path!("api" / "v1" / "portfolios" / u64)
                    .and(filters.clone())
                    .and(with_path_name("/api/v1/portfolios/{pid}".to_string()))
                    .and(with_param_name("pid".to_string()))
                    .and_then(with_param))
                .or(
                    warp::path!("api" / "v1" / "portfolios" / u64 / "relationships" / "securities")
                        .and(filters.clone())
                        .and(with_path_name(
                            "/api/v1/portfolios/{pid}/relationships/securities".to_string(),
                        ))
                        .and(with_param_name("pid".to_string()))
                        .and_then(with_param),
                )
                .or(warp::path!(
                    "api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64
                )
                .and(filters.clone())
                .and(with_path_name(
                    "/api/v1/portfolios/{pid}/securities/{sid}/transactions/{tid}".to_string(),
                ))
                .and(with_param_name("pid".to_string()))
                .and(with_param_name("sid".to_string()))
                .and(with_param_name("tid".to_string()))
                .and_then(with_3_params)),
        )
        .boxed();

    get.or(post).or(patch).or(delete).boxed()
}

fn with_publisher(
    publisher: Arc<RwLock<Publisher>>,
) -> impl Filter<Extract = (Arc<RwLock<Publisher>>,), Error = Infallible> + Clone {
    warp::any().map(move || publisher.clone())
}

fn with_router(
    r: Arc<Router>,
) -> impl Filter<Extract = (Arc<Router>,), Error = Infallible> + Clone {
    warp::any().map(move || r.clone())
}

fn with_path_name(name: String) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    warp::any().map(move || name.clone())
}

fn with_param_name(name: String) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    warp::any().map(move || name.clone())
}
