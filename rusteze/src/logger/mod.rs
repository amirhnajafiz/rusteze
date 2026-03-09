use tracing_subscriber::{ fmt, EnvFilter };

// Initialize the logger with the specified log level.
// The log level can be set via the RUST_LOG environment variable or defaults to "info".
pub fn init_logger(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}
