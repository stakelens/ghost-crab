use crate::config;
use crate::indexer::{run, HandlerConfig, HandlerFn};

#[derive(Clone)]
pub struct Indexer {
    config: config::Config,
    handlers: Vec<HandlerConfig>,
}

impl Indexer {
    pub fn new() -> Indexer {
        let config = config::load();

        return Indexer {
            config: config.clone(),
            handlers: Vec::new(),
        };
    }

    pub fn load(&mut self, handler: HandlerFn) {
        if handler.is_template() {
            return;
        }

        let source = self.config.data_sources.get(&handler.get_source()).unwrap();

        self.handlers.push(HandlerConfig {
            start_block: source.start_block,
            address: source.address.clone(),
            network: source.network.clone(),
            step: 10_000,
            handler,
        });
    }

    pub async fn start(self) {
        run(self.handlers).await;
    }
}
