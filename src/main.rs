use clap::{App, Arg};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};

fn main() {
    let matches = App::new("httpwsrep")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("DSN")
                .env("DSN")
                .help("mysql://<user>:<password>@unix(/tmp/mysql.sock)/<database>")
                .long("dsn")
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .default_value("9200")
                .help("HTTP port")
                .long("port")
                .required(true)
                .short("p")
                .validator(is_num),
        )
        .get_matches();

    let port = matches.value_of("port").unwrap().parse::<u16>().unwrap();

    // This is our socket address...
    let addr = ([127, 0, 0, 1], port).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let new_svc = || {
        // service_fn_ok converts our function into a `Service`
        service_fn_ok(hello_world)
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    // Run this server for... forever!
    hyper::rt::run(server);
}

const PHRASE: &str = "Hello, World!";

fn hello_world(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from(PHRASE))
}

fn is_num(s: String) -> Result<(), String> {
    if let Err(..) = s.parse::<u64>() {
        return Err(String::from("Not a valid number!"));
    }
    Ok(())
}
