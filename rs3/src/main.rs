// main.rs

use clap::Parser;
use std::{
    fs,
    io::{BufReader, Write, prelude::*},
    net::{Shutdown, TcpListener, TcpStream},
    process,
};
use tracing::{error, info, instrument, warn};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Server IP
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    host: String,

    /// Server port
    #[arg(long, default_value_t = 8080)]
    port: i32,
}

fn main() {
    // start log tracer (JSON logging)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    // parse args
    let args = Args::parse();

    // server address
    let address = &format!("{}:{}", args.host, args.port);

    // create a TCP router, exit if it fails
    let router = match TcpListener::bind::<&str>(address) {
        Ok(listener) => listener,
        Err(error) => {
            error!(error = error.to_string(), "failed to build router");
            process::exit(1);
        }
    };

    info!(host = args.host, port = args.port, "server start");

    // loop over the router for incoming traffic
    for stream in router.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(error) => {
                warn!(error = error.to_string(), "failed to accept connection");
                continue;
            }
        };

        handler(stream);
    }
}

#[instrument(skip_all)]
fn handler(mut stream: TcpStream) {
    // extract the input connection address
    let in_addr = stream.peer_addr().unwrap();
    info!(addr = in_addr.to_string(), "connection established");

    // get the request content
    let buffer = BufReader::new(&stream);
    let request: Vec<_> = buffer
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // this line should print the HTTP request content
    info!("{:#?}", request);

    // create an HTTP response
    let status_line = "HTTP/1.1 200 OK";
    let content = fs::read_to_string("index.html").unwrap();
    let length = content.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

    // write into the stream and close it
    stream.write_all(response.as_bytes()).unwrap();
    stream.shutdown(Shutdown::Both).unwrap();
}
