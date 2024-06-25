use crate::cache::manager::RPC_MANAGER;
use crate::config;
use crate::handler::{HandleInstance, HandlerConfig};
use crate::process_logs::process_logs;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Clone)]
pub struct TemplateManager {
    config: config::Config,
    tx: Sender<HandlerConfig>,
}

pub struct Template {
    pub start_block: u64,
    pub address: String,
    pub handler: HandleInstance,
}

impl TemplateManager {
    pub async fn start(&self, template: Template) {
        let source = self
            .config
            .templates
            .get(&template.handler.get_source())
            .unwrap();

        let provider = RPC_MANAGER.lock().await.get(source.network.clone()).await;

        self.tx
            .send(HandlerConfig {
                start_block: template.start_block,
                address: template.address.clone(),
                step: 10_000,
                provider,
                handler: template.handler,
                templates: Arc::new(self.clone()),
            })
            .await
            .unwrap();
    }
}

pub struct Indexer {
    config: config::Config,
    handlers: Vec<HandlerConfig>,
    rx: Receiver<HandlerConfig>,
    templates: Arc<TemplateManager>,
}

impl Indexer {
    pub fn new() -> Indexer {
        let config = config::load();
        let (tx, rx) = mpsc::channel::<HandlerConfig>(1);

        let templates = TemplateManager {
            config: config.clone(),
            tx,
        };

        return Indexer {
            config: config.clone(),
            handlers: Vec::new(),
            rx,
            templates: Arc::new(templates),
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
            templates: self.templates.clone(),
        });
    }

    pub async fn start(mut self) {
        for handler in self.handlers {
            tokio::spawn(async move {
                process_logs(handler).await;
            });
        }

        // For dynamic sources
        while let Some(handler) = self.rx.recv().await {
            tokio::spawn(async move {
                process_logs(handler).await;
            });
        }
    }
}
