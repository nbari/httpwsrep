use chrono::prelude::*;
use httpwsrep::{options, queries};
use lazy_static::lazy_static;
use prometheus::{Encoder, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, Opts, Registry};
use std::net::{IpAddr, Ipv4Addr};
use std::process;
use std::str::FromStr;
use tokio::time::{timeout, Duration};
use warp::filters::log::{Info, Log};
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref CONNECTION_ERROR: IntCounter =
        IntCounter::new("connection_error", "Connection error").expect("metric can be created");
    static ref WSREP_LOCAL_STATE: IntCounterVec =
        IntCounterVec::new(Opts::new("state", "Node State"), &["state"])
            .expect("metric can be created");
    static ref RESPONSE_TIME: HistogramVec = HistogramVec::new(
        HistogramOpts::new("response_time", "HTTP response times"),
        &["method", "handler"],
    )
    .expect("metric can be created");
}

fn metrics_end(handler_name: &'static str) -> Log<impl Fn(Info<'_>) + Copy> {
    warp::log::custom(move |info: Info<'_>| {
        let duration = info.elapsed().as_secs_f64();
        let method = info.method().clone();
        RESPONSE_TIME
            .with_label_values(&[method.as_str(), handler_name])
            .observe(duration);
    })
}

#[tokio::main]
async fn main() {
    REGISTRY
        .register(Box::new(CONNECTION_ERROR.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(WSREP_LOCAL_STATE.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(RESPONSE_TIME.clone()))
        .expect("collector can be registered");

    let (v46, port, pool) = options::new();
    if timeout(Duration::from_secs(3), pool.get_conn())
        .await
        .is_err()
    {
        eprintln!("Could not connect to the Galera node");
        process::exit(1);
    }

    let now = Utc::now();
    println!(
        "{} - Listening on *:{}",
        now.to_rfc3339_opts(SecondsFormat::Secs, true),
        port
    );

    let addr = if v46 {
        // tcp46 or fallback to tcp4
        match IpAddr::from_str("::0") {
            Ok(a) => a,
            Err(_) => IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        }
    } else {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    };

    let metrics = warp::path("metrics").and(warp::get().and_then(metrics_handler));

    let state = warp::path::end().and(
        warp::any()
            .map(move || pool.clone())
            .and_then(state_handler)
            .with(metrics_end("state")),
    );

    let routes = state.or(metrics);

    warp::serve(routes).run((addr, port)).await;
}

// state query database and if wsrep_local_state == 4 it will return HTTP 200
// OK, otherwise HTTP 503 Service Unavailable
async fn state_handler(pool: mysql_async::Pool) -> Result<impl warp::Reply, warp::Rejection> {
    let rs = match queries::state(pool.clone()).await {
        Ok(rs) => rs,
        Err(e) => {
            CONNECTION_ERROR.inc();
            eprintln!("{:?}", e);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match rs {
        4 => {
            WSREP_LOCAL_STATE.with_label_values(&["4"]).inc();
            Ok(StatusCode::OK)
        }
        n => {
            WSREP_LOCAL_STATE.with_label_values(&[&n.to_string()]).inc();
            Ok(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// # Errors
/// return Err if can't encode
pub async fn metrics_handler() -> Result<impl Reply, Rejection> {
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        eprintln!("could not encode custom metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();
    Ok(res)
}
