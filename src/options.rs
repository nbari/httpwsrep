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
pub fn new() -> (u16, mysql_async::Pool) {
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
            Arg::with_name("min")
                .default_value("3")
                .help("mysql pool min connections")
                .long("min")
                .validator(is_num),
        )
        .arg(
            Arg::with_name("max")
                .default_value("50")
                .help("mysql pool max connections")
                .long("max")
                .validator(is_num),
        )
        .arg(
            Arg::with_name("port")
                .default_value("9200")
                .help("listening port")
                .long("port")
                .validator(is_num)
                .required(true),
        )
        .get_matches();

    // prepare DSN for the mysql pool
    let dsn = matches.value_of("dsn").unwrap();
    let dsn = dsn::parse(dsn).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
    let pool_min = matches.value_of("min").unwrap().parse::<usize>().unwrap();
    let pool_max = matches.value_of("max").unwrap().parse::<usize>().unwrap();

    let mut opts = mysql_async::OptsBuilder::new();
    opts.user(dsn.username);
    opts.pass(dsn.password.clone());
    if let Some(host) = dsn.host {
        opts.ip_or_hostname(host);
        if let Some(port) = dsn.port {
            opts.tcp_port(port);
        }
    }
    opts.socket(dsn.socket);
    opts.db_name(dsn.database);
    opts.pool_options(mysql_async::PoolOptions::with_constraints(
        mysql_async::PoolConstraints::new(pool_min, pool_max).unwrap(),
    ));

    // mysql ssl options
    let mut ssl_opts = mysql_async::SslOpts::default();
    if let Some(tls) = dsn.params.get("tls") {
        if *tls == "skip-verify" {
            ssl_opts.set_danger_accept_invalid_certs(true);
        }
        opts.ssl_opts(ssl_opts);
    }

    opts.conn_ttl(Duration::new(60, 0));

    let port = matches.value_of("port").unwrap().parse::<u16>().unwrap();
    (port, mysql_async::Pool::new(opts))
}
