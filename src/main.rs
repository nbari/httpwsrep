extern crate clap;

use clap::{App, Arg};


fn main() {
    let matches = App::new("httpwsrep")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("socket")
             .short("s")
             .long("socket")
             .value_name("PATH")
             .help("path to mysql socket")
             .required(true))
        .get_matches();


    let socket = matches.value_of("socket").unwrap_or("/tmp/mysql.sock");
    println!("Value for socket: {}", socket);



}
