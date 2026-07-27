// main.rs

use std::{
    fs,
    io::{BufReader, Write, prelude::*},
    net::{Shutdown, TcpListener, TcpStream},
    process,
};

fn main() {
    // server address (TODO: read from a config file / argument flag)
    let address = "127.0.0.1:8080";

    // create a TCP router, exit if it fails
    let router = match TcpListener::bind::<&str>(address) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("failed to build router: {}", error);
            process::exit(1);
        }
    };

    println!("tpc server start: {} ...", address);

    // loop over the router for incoming traffic
    for stream in router.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(error) => {
                eprintln!("failed to accept connection: {}", error);
                continue;
            }
        };

        handler(stream);
    }
}

fn handler(mut stream: TcpStream) {
    // extract the input connection address
    let in_addr = stream.peer_addr().unwrap();
    println!("connection established: {}", in_addr);

    // get the request content
    let buffer = BufReader::new(&stream);
    let request: Vec<_> = buffer
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // this line should print the HTTP request content
    println!("request captured: {:#?}", request);

    // create an HTTP response
    let status_line = "HTTP/1.1 200 OK";
    let content = fs::read_to_string("index.html").unwrap();
    let length = content.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

    // write into the stream and close it
    stream.write_all(response.as_bytes()).unwrap();
    stream.shutdown(Shutdown::Both).unwrap();
}
