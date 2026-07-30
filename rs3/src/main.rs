// main.rs

mod loger;
mod workers;

use anyhow::Result;
use clap::Parser;
use std::{
    collections::HashMap,
    fs,
    io::{self, BufReader, ErrorKind, Write, prelude::*},
    net::{Shutdown, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process,
    sync::Arc,
    time::Duration,
};
use tracing::{error, info, instrument, warn};
use workers::WorkerPool;

const IO_TIMEOUT: Duration = Duration::from_secs(10);

// directory of templates
const TEMP_DIR: &str = "templates/";

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
    // init logger (must keep the guard for file writer)
    let _guard = loger::init_logger();

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

    // create a thread pool
    let pool = WorkerPool::new(4);

    // build the templates path
    let path = Path::new(".").join(TEMP_DIR);
    let mut templates: HashMap<&str, PathBuf> = HashMap::new();

    templates.insert("index", path.join("index.html"));
    templates.insert("404", path.join("404.html"));

    // shadow templates for read-only
    let templates = Arc::new(templates);

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

        let templates_clone = templates.clone();
        pool.execute(move || {
            handler(stream, &templates_clone);
        });
    }
}

#[instrument(skip_all)]
fn handler(mut stream: TcpStream, templates: &HashMap<&str, PathBuf>) {
    // a dead client must never be able to block a worker thread indefinitely
    if let Err(err) = stream.set_read_timeout(Some(IO_TIMEOUT)) {
        warn!(error = %err, "failed to set read timeout");
    }
    if let Err(err) = stream.set_write_timeout(Some(IO_TIMEOUT)) {
        warn!(error = %err, "failed to set write timeout");
    }

    let result = (|| -> Result<()> {
        // extract the input connection address
        let in_addr = stream.peer_addr()?;
        info!(addr = in_addr.to_string(), "connection established");

        // get the request content
        let request = {
            let buffer = BufReader::new(&stream);
            let mut lines = buffer.lines();

            match lines.next() {
                Some(line) => line?,
                None => {
                    return Err(anyhow::Error::new(io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "empty request",
                    )));
                }
            }
        };

        // this line should print the HTTP request content
        info!(request = request, "request");

        // routing logic
        let (status_line, content_path) = match request.as_str() {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", templates["index"].display()),
            _ => ("HTTP/1.1 404 NOT FOUND", templates["404"].display()),
        };

        // create an HTTP response
        let content = fs::read_to_string(content_path.to_string())?;
        let length = content.len();
        let response = format!(
            "{status_line}\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Content-Length: {length}\r\n\
             Connection: close\r\n\
             \r\n\
             {content}"
        );

        // write into the stream and close it
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();

        Ok(())
    })();

    // handle internal errors
    if let Err(err) = result {
        error!(error = %err, "request failed");
        let _ = stream.write_all(
            b"HTTP/1.1 500 Internal Server Error\r\n\
              Content-Length: 0\r\n\
              Connection: close\r\n\r\n",
        );
    }

    // write into the stream and close it
    if let Err(err) = stream.shutdown(Shutdown::Write) {
        warn!(error = %err, "failed to shutdown connection");
    }
}
