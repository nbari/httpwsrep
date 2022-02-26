#[macro_use]
extern crate lazy_static;

use chrono::prelude::*;
use httpwsrep::{options, queries};
use std::net::{IpAddr, Ipv4Addr};
use std::process;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use prometheus::{Encoder, IntCounter, IntCounterVec, Opts, Registry};

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref INCOMING_REQUESTS: IntCounter =
        IntCounter::new("incoming_requests", "Incoming Requests").expect("metric can be created");
    static ref CONNECTION_ERROR: IntCounter =
        IntCounter::new("connection_error", "Connection error").expect("metric can be created");
    static ref WSREP_LOCAL_STATE: IntCounterVec =
        IntCounterVec::new(Opts::new("state", "Node State"), &["state"])
            .expect("metric can be created");
}

#[tokio::main]
async fn main() {
    REGISTRY
        .register(Box::new(INCOMING_REQUESTS.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(CONNECTION_ERROR.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(WSREP_LOCAL_STATE.clone()))
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

    let db = warp::any().map(move || pool.clone());

    let state = warp::any().and(db.clone()).and_then(state);

    let addr = if v46 {
        // tcp46 or fallback to tcp4
        match IpAddr::from_str("::0") {
            Ok(a) => a,
            Err(_) => IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        }
    } else {
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    };

    let metrics_route = warp::path!("metrics").and_then(metrics_handler);
    warp::serve(metrics_route.or(state)).run((addr, port)).await;
}

// state query database and if wsrep_local_state == 4 it will return HTTP 200
// OK, otherwise HTTP 503 Service Unavailable
async fn state(pool: mysql_async::Pool) -> Result<impl warp::Reply, warp::Rejection> {
    INCOMING_REQUESTS.inc();
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

async fn metrics_handler() -> Result<impl Reply, Rejection> {
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        eprintln!("could not encode custom metrics: {}", e);
    };
    let mut res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
        eprintln!("could not encode prometheus metrics: {}", e);
    };
    let res_custom = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("prometheus metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    res.push_str(&res_custom);
    Ok(res)
}
