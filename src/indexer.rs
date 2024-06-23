use crate::cache::manager::RPC_MANAGER;
use crate::config;
use crate::handler::{HandleInstance, HandlerConfig};
use crate::process_logs::process_logs;

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

    pub async fn load(&mut self, handler: HandleInstance) {
        if handler.is_template() {
            return;
        }

        let source = self.config.data_sources.get(&handler.get_source()).unwrap();
        let provider = RPC_MANAGER.lock().await.get(source.network.clone()).await;

        self.handlers.push(HandlerConfig {
            start_block: source.start_block,
            address: source.address.clone(),
            step: 10_000,
            provider,
            handler,
        });
    }

    pub async fn start(self) {
        let join_handles = self
            .handlers
            .into_iter()
            .map(|process| {
                tokio::spawn(async move {
                    process_logs(process).await;
                })
            })
            .collect::<Vec<_>>();

        for handle in join_handles {
            handle.await.unwrap();
        }
    }
}
