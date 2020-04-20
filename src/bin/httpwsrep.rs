use chrono::prelude::*;
use httpwsrep::{options, queries};
use std::process;
use warp::http::StatusCode;
use warp::Filter;

#[tokio::main]
async fn main() {
    let (port, pool) = options::new();
    pool.get_conn().await.unwrap_or_else(|e| {
        eprintln!("Could not connect to MySQL: {}", e);
        process::exit(1);
    });

    let now = Utc::now();
    println!(
        "{} - Listening on *:{}",
        now.to_rfc3339_opts(SecondsFormat::Secs, true),
        port
    );

    let db = warp::any().map(move || pool.clone());

    let state = warp::get().and(db.clone()).and_then(state);

    warp::serve(state).run(([0, 0, 0, 0], port)).await;
}

async fn state(pool: mysql_async::Pool) -> Result<impl warp::Reply, warp::Rejection> {
    let rs = match queries::state(pool.clone()).await {
        Ok(rs) => rs,
        err => {
            eprintln!("{:?}", err);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    match rs {
        4 => Ok(StatusCode::OK),
        _ => Ok(StatusCode::SERVICE_UNAVAILABLE),
    }
}
