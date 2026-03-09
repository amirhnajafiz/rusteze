mod api;
mod configs;
mod logger;
mod memcache;
mod requests;
mod worker;

use tracing::{ info, error };

// init_env function to initialize the environment, such as creating necessary directories.
fn init_env(config: &configs::AppConfig) {
    // initialize the logger with the specified log level
    logger::init_logger(&config.log_level);

    // create the data directory if it doesn't exist
    std::fs::create_dir_all(&config.data_dir).unwrap_or_else(|e| {
        error!("failed to create data directory: {}", e);
        std::process::exit(1);
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load the configuration
    let app_config = match configs::load_config("config.yaml") {
        Ok(app_config) => app_config,
        Err(e) => {
            error!("failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // initialize the environment
    init_env(&app_config);

    // create the memcache instance to be shared across the API handlers
    let mem_cache = std::sync::Arc::new(tokio::sync::Mutex::new(memcache::MemCache::new()));

    // start the cleanup task to remove expired keys every minute
    let mem_cache_clone = mem_cache.clone();
    tokio::spawn(async move {
        worker::worker_memcache_cleanup(60, mem_cache_clone).await;
    });

    // print the host:port
    info!("server running on {}:{}", app_config.host, app_config.port);

    // create and start the API server
    let api_server = api::APIServer {
        mem_cache: mem_cache.clone(),
    };
    let addr = format!("{}:{}", app_config.host, app_config.port)
        .parse()
        .map_err(|e| {
            error!("failed to parse server address: {}", e);
            std::process::exit(1);
        })
        .unwrap();

    api_server.start(addr).await;
    Ok(())
}
