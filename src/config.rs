use dotenvy::dotenv;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::{env, fs};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub abi: String,
    pub network: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSource {
    pub abi: String,
    pub address: String,
    pub start_block: u64,
    pub network: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub database: String,
    pub data_sources: HashMap<String, DataSource>,
    pub templates: HashMap<String, Template>,
    pub networks: HashMap<String, String>,
}

static CONFIG_CACHE: Lazy<Config> = Lazy::new(|| {
    dotenv().ok();
    let current_dir = env::current_dir().unwrap();
    let config_string = fs::read_to_string(current_dir.join("config.json")).unwrap();
    let mut config: Config = serde_json::from_str(&config_string).unwrap();

    if config.database.starts_with("$") {
        config.database = env::var(&config.database[1..]).unwrap();
    }

    config.networks.iter_mut().for_each(|(_key, value)| {
        if value.starts_with("$") {
            *value = env::var(&value[1..]).unwrap();
        }
    });

    config
});

pub fn load() -> Config {
    return CONFIG_CACHE.clone();
}
