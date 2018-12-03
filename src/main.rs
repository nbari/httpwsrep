extern crate clap;

use clap::{App, Arg};
// use mysql::{Pool, Opts};
use mysql::Pool;

extern crate mysql;

fn main() {
    let matches = App::new("httpwsrep")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("host")
             .short("h")
             .long("host")
             .value_name("localhost")
             .help("mysql host"))
        .arg(Arg::with_name("port")
             .short("P")
             .long("port")
             .value_name("3306")
             .help("mysql port"))
        .arg(Arg::with_name("user")
             .short("u")
             .long("user")
             .value_name("root")
             .help("mysql user"))
        .arg(Arg::with_name("password")
             .short("p")
             .long("password")
             .value_name("password")
             .help("mysql password"))
        .get_matches();


    let host = matches.value_of("host").unwrap_or("localhost");
    let port = matches.value_of("port").unwrap_or("port");
    let username = matches.value_of("user").unwrap_or("root");
    let password = matches.value_of("password").unwrap_or("");
    println!("Value for socket: {} {} {} {}", host, port, username, password);


}
