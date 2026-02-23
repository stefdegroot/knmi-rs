use serde_derive::Deserialize;
use std::{fs, env};
use std::process::exit;
use toml;
use lazy_static::lazy_static;
use crate::knmi::sources::KnmiSourceTag;

#[derive(Deserialize, Debug, Clone)]
pub struct Server {
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenDataApi {
    pub token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NotificationService {
    pub url: String,
    pub port: u16,
    pub token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Knmi {
    pub open_data_api: OpenDataApi,
    pub notification_service: NotificationService,
    pub sources: Vec<KnmiSourceTag>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server: Server,
    pub knmi: Knmi,
}

pub fn load_config () -> Config {

    let mut path = env::current_dir().unwrap();
    path.push("config.toml");

    println!("{:?}", path);

    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            tracing::error!("Could not find config.toml file.");
            exit(1);
        }
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(err) => {
            tracing::error!("Could not read config.toml file.");
            tracing::error!("{:?}", err);
            exit(1);
        }
    };

    config
}

lazy_static! {
    pub static ref CONFIG: Config = {
        load_config()
    };
}