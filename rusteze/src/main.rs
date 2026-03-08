mod configs;

fn main() {
    // load the configuration
    let app_config = match configs::load_config() {
        Ok(app_config) => {
            app_config
        }
        Err(e) => {
            eprintln!("failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // print the host:port 
    println!("server will run on {}:{}", app_config.host, app_config.port);
}
