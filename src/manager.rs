use crate::indexer::{run, Config as RunConfig, DataSourceConfig, Handleable};
use dotenvy::dotenv;
use serde::Deserialize;
use std::collections::HashMap;
use std::{env, fs};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataSource {
    abi: String,
    address: String,
    start_block: u64,
    network: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    database: String,
    sources: HashMap<String, DataSource>,
    networks: HashMap<String, String>,
}

pub struct Indexer {
    config: Config,
    data_sources: Vec<DataSourceConfig>,
}

impl Indexer {
    pub fn load() -> Indexer {
        dotenv().ok();

        let config_string = fs::read_to_string("./config.json").unwrap();
        let mut config: Config = serde_json::from_str(&config_string).unwrap();

        if config.database.starts_with("$") {
            config.database = env::var(&config.database[1..]).unwrap()
        }

        config
            .networks
            .iter_mut()
            .for_each(|(_key, value)| {
                if value.starts_with("$") {
                    *value = env::var(&value[1..]).unwrap();
                }
            });

        return Indexer {
            config,
            data_sources: Vec::new(),
        };
    }

    pub fn add(&mut self, handler: Box<(dyn Handleable + Send + Sync)>) {
        let handler_data_source = self.config.sources.get(&handler.get_data_source()).unwrap();
        let rpc_url = self
            .config
            .networks
            .get(&handler_data_source.network)
            .unwrap();

        self.data_sources.push(DataSourceConfig {
            start_block: handler_data_source.start_block,
            step: 10_000,
            address: handler_data_source.address.clone(),
            handler: handler,
            rpc_url: rpc_url.clone(),
        });
    }

    pub async fn start(self) {
        run(RunConfig {
            db_url: self.config.database,
            data_sources: self.data_sources,
        })
        .await;
    }
}
