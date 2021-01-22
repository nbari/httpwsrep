use clap::{App, Arg};
use std::process;
use std::time::Duration;

fn is_num(s: String) -> Result<(), String> {
    if let Err(..) = s.parse::<usize>() {
        return Err(String::from("Not a valid number!"));
    }
    Ok(())
}

#[must_use]
// returns (v46, port, pool)
pub fn new() -> (bool, u16, mysql_async::Pool) {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("dsn")
                .env("DSN")
                .help("mysql://<username>:<password>@tcp(<host>:<port>)/<database>")
                .long("dsn")
                .short("d")
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .default_value("9200")
                .help("listening port")
                .long("port")
                .validator(is_num)
                .required(true),
        )
        .arg(
            Arg::with_name("v46")
                .help("listen in both IPv4 and IPv6")
                .long("46"),
        )
        .get_matches();

    // prepare DSN for the mysql pool
    let dsn = matches.value_of("dsn").unwrap();
    let dsn = dsn::parse(dsn).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });

    let opts = mysql_async::OptsBuilder::default()
        .user(dsn.username)
        .pass(dsn.password)
        .db_name(dsn.database)
        .ip_or_hostname(dsn.host.unwrap_or_else(|| String::from("127.0.0.1")))
        .tcp_port(dsn.port.unwrap_or(3306))
        .socket(dsn.socket)
        .conn_ttl(Duration::new(60, 0));

    // mysql ssl options
    if let Some(tls) = dsn.params.get("tls") {
        if *tls == "skip-verify" {
            mysql_async::SslOpts::default().with_danger_accept_invalid_certs(true);
        }
    }

    let port = matches.value_of("port").unwrap().parse::<u16>().unwrap();
    (
        matches.is_present("v46"),
        port,
        mysql_async::Pool::new(opts),
    )
}
