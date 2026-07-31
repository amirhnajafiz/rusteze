// loger.rs

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{Registry, filter, fmt, prelude::*};

/// log path consts
const LOG_DIR: &str = "./logs";
const LOG_FILENAME: &str = "app.logs";

/// init logger sets our global tracing subscriber to export logs
/// into both a log file and stdout.
pub fn init_logger() -> WorkerGuard {
    // create a daily log rotator
    let file_appender = tracing_appender::rolling::daily(LOG_DIR, LOG_FILENAME);

    // wrap it in a non-blocking writer
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // create a json formatter
    let file_layer_format = fmt::format().json();

    // create log level filter
    let filter = filter::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| filter::EnvFilter::new("info"));

    // create a file layer
    let file_layer = fmt::Layer::default()
        .event_format(file_layer_format)
        .with_writer(file_writer)
        .json()
        .with_filter(filter.clone());

    // create a stdout layer
    let stdout_layer = fmt::Layer::default()
        .with_writer(std::io::stdout)
        .with_ansi(false)
        .with_filter(filter);

    // create a subscriber
    let subscriber = Registry::default().with(file_layer).with(stdout_layer);

    // register subscriber
    tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");

    guard
}
