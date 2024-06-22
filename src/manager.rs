use crate::config;
use crate::indexer::{run, DataSourceConfig, Handler, RunInput};
use std::sync::Arc;

pub struct Indexer {
    config: config::Config,
    data_sources: Vec<DataSourceConfig>,
}

impl Indexer {
    pub fn new() -> Indexer {
        let config = config::load();

        return Indexer {
            config: config.clone(),
            data_sources: Vec::new(),
        };
    }

    pub fn load(&mut self, handler: Box<(dyn Handler + Send + Sync)>) {
        if handler.is_template() {
            return;
        }

        let source = self.config.data_sources.get(&handler.get_source()).unwrap();

        self.data_sources.push(DataSourceConfig {
            start_block: source.start_block,
            address: source.address.clone(),
            network: source.network.clone(),
            step: 10_000,
            handler: Arc::new(handler),
        });
    }

    pub async fn start(self) {
        let database = self.config.database.clone();

        run(RunInput {
            data_sources: self.data_sources,
            database,
        })
        .await;
    }
}
