// #![deny(warnings)]
#![recursion_limit = "192"]

use std::collections::HashMap;
use std::convert::Infallible;

use deadpool_lapin::{Config, Pool, Runtime};
use env_logger::{Builder, Target};
use futures::join;
use log::LevelFilter;
use warp::{Filter, route};

mod handlers;
mod responses;
mod events;

pub struct Publisher<'a> {
    pool: &'a Pool,
}

impl<'a> Publisher<'a> {
    pub fn new(pool: &Pool) -> Publisher {
        Publisher {
            pool,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();

    builder.target(Target::Stdout);
    builder.filter_level(LevelFilter::Debug);
    builder.init();

    const BODY_SIZE: u64 = 1024 * 250;

    let mut cfg = Config::default();

    cfg.url = Some("amqp://user:pass@127.0.0.1:5672/%2f".into());

    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    let publ = Publisher::new(&pool);

    let filters = warp::any().
        and(route()).
        and(warp::header::optional::<String>("authorization")).
        and(warp::query::<HashMap<String, String>>()).
        and(with_rmq(pool.clone()));

    let body_filters = filters.clone().
        and(warp::body::bytes());

    let get = warp::get().
        and(
            warp::path!("api" / "v1" / "statuses").and_then(handlers::health_check).
                or(warp::path!("api" / "v1" / "users"/ u64 / "news").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/users/{uid}/news".to_string())).
                    and(with_param("uid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "users"/ u64 / "earnings").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/users/{uid}/earning".to_string())).
                    and(with_param("uid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "users"/ u64 / "dividends").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/users/{uid}/dividends".to_string())).
                    and(with_param("uid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "users"/ u64 / "day-prices").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/users/{uid}/day-prices".to_string())).
                    and(with_param("uid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "users"/ u64 / "day-price-periods").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/users/{uid}/day-prices-periods".to_string())).
                    and(with_param("uid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "users"/ u64 / "view-history").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/users/{uid}/view-history".to_string())).
                    and(with_param("uid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "users"/ u64).
                    and(filters.clone()).
                    and(with_path_name("/api/v1/users/{uid}".to_string())).
                    and(with_param("uid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "confirmation-codes").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/confirmation-codes".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "plans").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/plans".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "countries").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/countries".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "currencies").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/currencies".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "sectors").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/sectors".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "industries").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/industries".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "exchanges").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/exchanges".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "portfolios").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "portfolios" / u64).
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string())).
                    and(with_param("pid".to_string())).
                    and(with_param("sid".to_string())).
                    and_then(handlers::with_2_params)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64).
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string())).
                    and(with_param("pid".to_string())).
                    and(with_param("sid".to_string())).
                    and(with_param("tid".to_string())).
                    and_then(handlers::with_3_params)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "securities").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/securities".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "news").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/news".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "earnings").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/earnings".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "dividends").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/dividends".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "day-prices").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/day-prices".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "day-price-periods").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/day-price-periods".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "securities").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities".to_string())).
                    and_then(handlers::handle)).
                or(warp::path!("api" / "v1" / "securities" / u64 / "news").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities/{sid}/news".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "securities" / u64 / "day-prices").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities/{sid}/day-prices".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "securities" / u64 / "day-price-periods").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities/{sid}/day-price-periods".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "securities" / u64 / "quarterly-balance-sheet").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities/{sid}/quarterly-balance-sheet".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "securities" / u64 / "annual-balance-sheet").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities/{sid}/quarterly-balance-sheet".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "securities" / u64 / "quarterly-income-statements").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities/{sid}/quarterly-income-statements".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "securities" / u64 / "annual-income-statements").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities/{sid}/annual-income-statements".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "securities" / u64).
                    and(filters.clone()).
                    and(with_path_name("/api/v1/securities/{sid}".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)),
        );

    let post = warp::post().
        and(warp::body::content_length_limit(BODY_SIZE)).
        and(
            warp::path!("api" / "v1" / "users").
                and(body_filters.clone()).
                and(with_path_name("/api/v1/users".to_string())).
                and_then(handlers::with_body).
                or(warp::path!("api" / "v1" / "refresh-tokens").
                    and(body_filters.clone()).
                    and(with_path_name("/api/v1/refresh-tokens".to_string())).
                    and_then(handlers::with_body)).
                or(warp::path!("api" / "v1" / "sessions").
                    and(body_filters.clone()).
                    and(with_path_name("/api/v1/sessions".to_string())).
                    and_then(handlers::with_body)).
                or(warp::path!("api" / "v1" / "confirmation-codes" / String).
                    and(body_filters.clone()).
                    and(with_path_name("/api/v1/confirmation-codes/{id}".to_string())).
                    and(with_param("id".to_string())).
                    and_then(handlers::with_body_and_param)).
                or(warp::path!("api" / "v1" / "password-confirmation-codes").
                    and(body_filters.clone()).
                    and(with_path_name("/api/v1/password-confirmation-codes".to_string())).
                    and_then(handlers::with_body)).
                or(warp::path!("api" / "v1" / "portfolios").
                    and(body_filters.clone()).
                    and(with_path_name("/api/v1/portfolios".to_string())).
                    and_then(handlers::with_body)).
                or(warp::path!("api" / "v1" / "portfolios"/ u64 / "relationships" / "securities").
                    and(body_filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/relationships/securities".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_body_and_param)).
                or(warp::path!("api" / "v1" / "portfolios"/ u64 / "securities" / u64 / "transactions").
                    and(body_filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/securities/{sid}/transactions".to_string())).
                    and(with_param("pid".to_string())).
                    and(with_param("sid".to_string())).
                    and_then(handlers::with_body_and_2_params)),
        );

    let patch = warp::patch().and(warp::body::content_length_limit(BODY_SIZE)).and(
        warp::path!("api" / "v1" / "portfolios"/ u64).
            and(body_filters.clone()).
            and(with_path_name("/api/v1/portfolios/{pid}".to_string())).
            and(with_param("pid".to_string())).
            and_then(handlers::with_body_and_param).
            or(warp::path!("api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64).
                and(body_filters.clone()).
                and(with_path_name("/api/v1/portfolios/{pid}/securities/{sid}/transactions/{tid}".to_string())).
                and(with_param("pid".to_string())).
                and(with_param("sid".to_string())).
                and(with_param("tid".to_string())).
                and_then(handlers::with_body_and_3_params))
    );

    let delete = warp::delete().
        and(
            warp::path!("api" / "v1" / "refresh-tokens"/ u64).
                and(filters.clone()).
                and(with_path_name("/api/v1/refresh-tokens/{refresh-token}".to_string())).
                and(with_param("refresh-token".to_string())).
                and_then(handlers::with_param).
                or(warp::path!("api" / "v1" / "portfolios"/ u64).
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "portfolios"/ u64 / "relationships" / "securities").
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/relationships/securities".to_string())).
                    and(with_param("pid".to_string())).
                    and_then(handlers::with_param)).
                or(warp::path!("api" / "v1" / "portfolios" / u64 / "securities" / u64 / "transactions" / u64).
                    and(filters.clone()).
                    and(with_path_name("/api/v1/portfolios/{pid}/securities/{sid}/transactions/{tid}".to_string())).
                    and(with_param("pid".to_string())).
                    and(with_param("sid".to_string())).
                    and(with_param("tid".to_string())).
                    and_then(handlers::with_3_params)),
        );

    let routes = get.or(post).or(patch).or(delete);

    let _ = join!(
        warp::serve(routes).run(([127, 0, 0, 1], 3030)),
    );

    Ok(())
}

fn with_rmq(pool: Pool) -> impl Filter<Extract=(Pool, ), Error=Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn with_path_name(name: String) -> impl Filter<Extract=(String, ), Error=Infallible> + Clone {
    warp::any().map(move || name.clone())
}

fn with_param(name: String) -> impl Filter<Extract=(String, ), Error=Infallible> + Clone {
    warp::any().map(move || name.clone())
}
