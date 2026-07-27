// main.rs

use std::io::Write;
use std::net::{Shutdown, TcpListener};
use std::process;

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
        let mut stream = match stream {
            Ok(s) => s,
            Err(error) => {
                eprintln!("failed to accept connection: {}", error);
                continue;
            }
        };

        let in_addr = stream.peer_addr().unwrap();
        println!("connection established: {}", in_addr);

        stream.write(b"hello client").unwrap();
        stream.shutdown(Shutdown::Both).unwrap();
    }
}
