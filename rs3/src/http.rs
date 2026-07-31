// http.rs

use crate::workers::WorkerPool;
use ::std::{
    collections::HashMap,
    fs,
    io::{self, BufReader, ErrorKind, Write, prelude::*},
    net::{Shutdown, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use anyhow::Result;
use tracing::{error, info, instrument, warn};

// directory of templates
const TEMP_DIR: &str = "templates/";

// network timeout
const IO_TIMEOUT: Duration = Duration::from_secs(10);

/// HTTPHandler holds the traffic routing logic.
pub struct HTTPHandler {
    address: String,
    templates: Arc<HashMap<String, PathBuf>>,
}

impl HTTPHandler {
    #[instrument(skip_all)]
    pub fn new(host: &str, port: &str) -> HTTPHandler {
        // build the templates path
        let path = Path::new(".").join(TEMP_DIR);
        let mut templates: HashMap<String, PathBuf> = HashMap::new();

        templates.insert(String::from("index"), path.join("index.html"));
        templates.insert(String::from("404"), path.join("404.html"));

        // shadow templates for read-only
        let templates = Arc::new(templates);

        HTTPHandler {
            address: format!("{}:{}", host, port),
            templates,
        }
    }

    #[instrument(skip_all)]
    pub fn listen_and_serve(self: Arc<Self>) -> Result<()> {
        info!(address = self.address, "server start");

        // create a TCP router, exit if it fails
        let router = TcpListener::bind::<&str>(&self.address.to_string())?;

        // create a thread pool
        let pool = WorkerPool::new(4);

        // loop over the router for incoming traffic
        for stream in router.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(error) => {
                    warn!(error = error.to_string(), "failed to accept connection");
                    continue;
                }
            };

            let self_clone = self.clone();
            pool.execute(move || {
                self_clone.handler(stream);
            });
        }

        Ok(())
    }

    #[instrument(skip_all)]
    fn handler(&self, mut stream: TcpStream) {
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
                "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", self.templates["index"].display()),
                _ => ("HTTP/1.1 404 NOT FOUND", self.templates["404"].display()),
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
            stream.write_all(response.as_bytes())?;
            stream.flush()?;

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
}
