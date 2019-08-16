//! `OpenDataSH_twitter_notifier` is a crate that posts removed or added Datasets from the OpenData Portal SH to Twitter
//!
//! It currently requires nightly, but should work on stable once the `async_await` feature is
//! stabilized.
#![feature(async_await)]
#![allow(non_snake_case)]
#![warn(missing_debug_implementations, missing_docs)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use clap::App;
use futures_locks::Mutex;
use futures_util::compat::Future01CompatExt;
use log::Level;
use tokio::timer::Interval;

use crate::ckan_api::CkanAPI;
use crate::config::Config;
use crate::twitter::LoginStatus::LoggedIn;
use crate::twitter::Twitter;

mod ckan_api;
mod config;
mod twitter;

#[tokio::main]
async fn main() -> Result<(), egg_mode::error::Error> {
    simple_logger::init_with_level(Level::Debug).unwrap();

    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(config_path) = matches.value_of("config") {
        info!("A config file was passed in: {}", config_path);
        if Path::new(config_path).exists() {
            crawl_api(config_path).await?;
            Ok(())
        } else {
            error!("Missing config");
            Ok(())
        }
    } else {
        error!("Missing config");
        Ok(())
    }
}

async fn crawl_api(config_path: &str) -> Result<(), egg_mode::error::Error> {
    let twitter = Twitter::new(Config::new(config_path));
    let twitter = Arc::new(Mutex::new(twitter));
    let twitter1 = twitter.clone();
    let mut pure_twitter = twitter1.lock().compat().await.expect("Unable to get lock");
    pure_twitter.login().await?;
    info!("login done");

    let twitter4 = twitter.clone();
    CkanAPI::new(twitter4).getPackageList().await;
    /*tokio::spawn(async move {
        CkanAPI::new(twitter4).getPackageList().await;
    });*/

    let mut interval = Interval::new_interval(Duration::from_secs(60 * 60));
    loop {
        let instant = interval.next().await;
        let twitter2 = twitter.clone();

        info!("fire; instant={:?}", instant);

        tokio::spawn(async {
            CkanAPI::new(twitter2).getPackageList().await;
        });
    }
}
