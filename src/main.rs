#![allow(non_snake_case)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::App;
use futures::future::Future;
use futures::stream::Stream;
use log::Level;
use tokio::timer::Interval;

use crate::ckan_api::CkanAPI;
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

    let twitter = Arc::new(Mutex::new(Twitter::new(config_struct)));
    twitter.lock().unwrap().login();
    debug!("{:?}", twitter.lock().unwrap().status());
    let global_twitter = twitter.clone();

    let task = Interval::new_interval(Duration::from_secs(60 * 60))
        .for_each(move |instant| {
            info!("fire; instant={:?}", instant);
            let foreach_twitter = global_twitter.clone();

            let api = CkanAPI::new();
            tokio::spawn(
                api.getPackageList()
                    .and_then(move |data| {
                        let andthen_twitter = foreach_twitter.clone();
                        info!("{:?}", data.result);
                        let mut added_datasets: Vec<String> = vec![];
                        let mut removed_datasets: Vec<String> = vec![];
                        if !Path::new("./data/").exists() {
                            fs::create_dir_all("./data/");
                        }
                        if Path::new("./data/latestPackageList.json").exists() {
                            let cache_file: &str =
                                &fs::read_to_string("./data/latestPackageList.json").unwrap();
                            let cache: Vec<String> = serde_json::from_str(cache_file).unwrap();
                            for value in &cache {
                                if !data.result.contains(&value) {
                                    removed_datasets.push(value.to_owned());
                                    info!("removed: {}", value)
                                }
                            }
                            for value in &data.result {
                                if !cache.contains(&value) {
                                    added_datasets.push(value.to_owned());
                                    info!("added: {}", value)
                                }
                            }
                        }
                        let serialized = serde_json::to_string(&data.result).unwrap();
                        fs::write("./data/latestPackageList.json", serialized)
                            .expect("Unable to write latestPackageList");

                        andthen_twitter
                            .lock()
                            .unwrap()
                            .post_changed_datasets(added_datasets, removed_datasets);
                        Ok(())
                    })
                    .map_err(|e| error!("{0}", e)),
            );
            Ok(())
        })
        .map_err(|e| error!("interval errored; err={:?}", e));

    tokio::run(task);
}
