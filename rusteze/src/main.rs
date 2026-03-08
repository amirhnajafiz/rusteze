mod configs;
mod api;
mod requests;

// init_env function to initialize the environment, such as creating necessary directories.
fn init_env(config: &configs::AppConfig) {
    // create the data directory if it doesn't exist
    std::fs::create_dir_all(&config.data_dir).unwrap_or_else(|e| {
        eprintln!("failed to create data directory: {}", e);
        std::process::exit(1);
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load the configuration
    let app_config = match configs::load_config("config.yaml") {
        Ok(app_config) => app_config,
        Err(e) => {
            eprintln!("failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // initialize the environment
    init_env(&app_config);

    // print the host:port 
    println!("server will run on {}:{}", app_config.host, app_config.port);

    // create and start the API server
    let api_server = api::APIServer {};
    let addr = format!("{}:{}", app_config.host, app_config.port).parse().map_err(|e| {
        eprintln!("failed to parse server address: {}", e);
        std::process::exit(1);
    }).unwrap();
    
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| {
            eprintln!("failed to create Tokio runtime: {}", e);
            std::process::exit(1);
        })
        .unwrap()
        .block_on(api_server.start(addr));
    Ok(())
}
