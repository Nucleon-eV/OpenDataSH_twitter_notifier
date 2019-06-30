#![allow(non_snake_case)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use std::error::Error;
use std::path::Path;

use clap::App;
use log::Level;

use crate::config::Config;
use crate::twitter::Twitter;

mod config;
mod twitter;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(Level::Debug).unwrap();

    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(config_path) = matches.value_of("config") {
        println!("A config file was passed in: {}", config_path);
        if Path::new(config_path).exists() {
            // Read config
            let config_struct = Config::new(config_path);

            let mut twitter = Twitter::new(config_struct);
            twitter.login();
        } else {
            error!("Missing config");
        }
    } else {
        panic!("Missing config");
    }

    Ok(())
}
