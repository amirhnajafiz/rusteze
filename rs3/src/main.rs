// main.rs

mod http;
mod logger;
mod workers;

use clap::Parser;
use std::sync::Arc;
use tracing::error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Server IP
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    host: String,

    /// Server port
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

fn main() {
    // init logger (must keep the guard for file writer)
    let _guard = logger::init_logger();

    // parse args
    let args = Args::parse();

    // create new HTTP handler
    let handler = http::HTTPHandler::new(args.host, args.port);
    let handler = Arc::new(handler);

    // start the handler
    if let Err(err) = handler.listen_and_serve() {
        error!(error = %err, "failed to start the HTTP handler");
    }
}
