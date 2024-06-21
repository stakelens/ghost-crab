use crate::config;
use crate::indexer::{run, DataSourceConfig, Handler, RunInput};
use std::sync::Arc;

pub struct Indexer {
    config: config::Config,
    data_sources: Vec<DataSourceConfig>,
    templates: DynamicHandlerManager,
}

impl Indexer {
    pub fn new() -> Indexer {
        let config = config::load();

        return Indexer {
            config: config.clone(),
            data_sources: Vec::new(),
            templates: DynamicHandlerManager::new(config),
        };
    }

    pub fn load(&mut self, handler: Box<(dyn Handler + Send + Sync)>) {
        if handler.is_template() {
            return;
        }

        let source = self.config.data_sources.get(&handler.get_source()).unwrap();
        let rpc_url = self.config.networks.get(&source.network).unwrap();

        self.data_sources.push(DataSourceConfig {
            start_block: source.start_block,
            step: 10_000,
            address: source.address.clone(),
            handler: Arc::new(handler),
            rpc_url: rpc_url.clone(),
        });
    }

    pub async fn start(self) {
        let database = self.config.database.clone();

        run(RunInput {
            data_sources: self.data_sources,
            database,
            dynamic_handler_manager: self.templates.clone(),
        })
        .await;
    }
}

#[derive(Clone)]
pub struct DynamicHandlerManager {
    config: config::Config,
}

impl DynamicHandlerManager {
    pub fn new(config: config::Config) -> Self {
        DynamicHandlerManager { config }
    }

    pub fn start(
        &self,
        handler: Box<(dyn Handler + Send + Sync)>,
        address: &str,
        start_block: u64,
    ) {
        let source = self.config.templates.get(&handler.get_source()).unwrap();
        let rpc_url = self.config.networks.get(&source.network).unwrap();

        let run_input = RunInput {
            database: self.config.database.clone(),
            data_sources: vec![DataSourceConfig {
                start_block: start_block,
                step: 10_000,
                address: String::from(address),
                handler: Arc::new(handler),
                rpc_url: rpc_url.clone(),
            }],
            dynamic_handler_manager: self.clone(),
        };

        tokio::spawn(async move {
            run(run_input).await;
        });
    }
}
