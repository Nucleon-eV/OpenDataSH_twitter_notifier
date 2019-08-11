#![allow(non_snake_case)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::time::Duration;

use clap::App;
use futures::future::Future;
use futures::Stream;
use log::Level;
use tokio::timer::Interval;

use crate::ckan_api::{CkanAPI, GetPackageList};
use crate::config::Config;
use crate::twitter::Twitter;

mod ckan_api;
mod config;
mod twitter;

#[allow(unreachable_code)]
fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(Level::Debug).unwrap();

    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(config_path) = matches.value_of("config") {
        info!("A config file was passed in: {}", config_path);
        if Path::new(config_path).exists() {
            crawl_api(config_path);
        } else {
            error!("Missing config");
        }
    } else {
        error!("Missing config");
    }

    Ok(())
}

fn crawl_api(config_path: &str) {
    // Read config
    let config_struct = Config::new(config_path);

    let mut twitter = Box::pin(Twitter::new(config_struct));
    twitter.login();
    debug!("{:?}", twitter.status());

    let api_task = GetPackageList {
        response: CkanAPI::new().getPackageList(),
        twitter,
    };

    tokio::run(api_task);

    let task = Interval::new_interval(Duration::from_secs(60 * 60))
        .for_each(move |instant| {
            info!("fire; instant={:?}", instant);

            let api_taskL = GetPackageList {
                response: CkanAPI::new().getPackageList(),
                twitter,
            };

            tokio::spawn(api_taskL);
            Ok(())
        })
        .map_err(|e| error!("interval errored; err={:?}", e));

    tokio::run(task);
}
