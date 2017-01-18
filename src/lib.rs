#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate hyper;
extern crate habitat_core;
extern crate habitat_depot_client;
extern crate habitat_common;
extern crate rustc_serialize;

pub mod config;
pub mod origin;
pub mod package;

use config::Config;
use std::io::Error;
use habitat_core::package::PackageIdent;

pub fn run(config: Config) -> Result<(), Box<Error>> {
    println!("Source: {:?}", config.source);
    println!("Destination: {:?}", config.destination);
    for origin in config.origins {
        println!("Processing origin: {}", origin.name);
        // for package in origin.packages.unwrap() {
        //    println!("--Package: {}", package.name);
        // }
        let source_package_list = origin.get_package_list(&config.source)
            .unwrap();
        let dest_package_list = origin.get_package_list(&config.destination).unwrap();
        let missing_packages = get_package_list_diff(source_package_list, dest_package_list);
        let archives = origin.download_packages(&config.source, missing_packages, "/tmp/habcache");

        origin.upload_packages(&config.destination,
                               archives,
                               "/tmp/habcache",
                               &config.token);
    }
    Ok(())
}

fn get_package_list_diff(source_list: Vec<PackageIdent>,
                         dest_list: Vec<PackageIdent>)
                         -> Vec<PackageIdent> {
    let mut result: Vec<PackageIdent> = vec![];
    for package in source_list {
        if (!dest_list.contains(&package)) {
            result.push(package);
        }
    }
    result
}
