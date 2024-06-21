use crate::config;
use crate::indexer::{run, DataSourceConfig, Handler, RunInput};
use std::collections::HashMap;

pub struct TemplateRun {
    pub handler: Box<dyn Handler + Send + Sync>,
    pub rpc_url: String,
}

pub struct Indexer {
    pub config: config::Config,
    pub data_sources: Vec<DataSourceConfig>,
    pub templates: TemplateManager,
}

impl Indexer {
    pub fn new() -> Indexer {
        let config = config::load();

        return Indexer {
            config: config.clone(),
            data_sources: Vec::new(),
            templates: TemplateManager::new(config),
        };
    }

    pub fn load(&mut self, handler: Box<(dyn Handler + Send + Sync)>) {
        if handler.is_template() {
            self.templates.add_handler(handler);
        } else {
            let source = self.config.data_sources.get(&handler.get_source()).unwrap();
            let rpc_url = self.config.networks.get(&source.network).unwrap();

            self.data_sources.push(DataSourceConfig {
                start_block: source.start_block,
                step: 10_000,
                address: source.address.clone(),
                handler: handler,
                rpc_url: rpc_url.clone(),
            });
        }
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

struct TemplateManager {
    pub config: config::Config,
    pub templates: HashMap<String, TemplateRun>,
}

impl TemplateManager {
    pub fn new(config: config::Config) -> Self {
        TemplateManager {
            config,
            templates: HashMap::new(),
        }
    }

    fn add_handler(&mut self, handler: Box<(dyn Handler + Send + Sync)>) {
        let source = self.config.templates.get(&handler.get_source()).unwrap();
        let rpc_url = self.config.networks.get(&source.network).unwrap();

        self.templates.insert(
            handler.get_source(),
            TemplateRun {
                handler: handler,
                rpc_url: rpc_url.clone(),
            },
        );
    }

    pub fn start(&mut self, name: &str, address: &str, start_block: u64) {
        let template = self.templates.remove(name).unwrap();

        let run_input = RunInput {
            database: self.config.database.clone(),
            data_sources: vec![DataSourceConfig {
                start_block: start_block,
                step: 10_000,
                address: String::from(address),
                handler: template.handler,
                rpc_url: template.rpc_url.clone(),
            }],
        };

        tokio::spawn(async move {
            run(run_input).await;
        });
    }
}
