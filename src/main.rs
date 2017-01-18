extern crate depot_sync;

use depot_sync::config::Config;
use std::env;
use std::process;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut config_path = String::new();
    if args.len() > 1 {
        config_path = args[1].clone();
    } else {
        config_path = "config/config.yml".to_string();
    }

    let config = Config::from_file(config_path).unwrap_or_else(|err| {
        println!("Could not read config file");
        process::exit(1);
    });

    if let Err(e) = depot_sync::run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
