use crate::block_handler::{process_logs_block, BlockHandlerInstance, ProcessBlocksInput};
use crate::cache::manager::RPCManager;
use crate::handler::HandleInstance;
use crate::process_logs::{process_logs, ProcessEventsInput};
use alloy::primitives::Address;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct Template {
    pub start_block: u64,
    pub address: Address,
    pub handler: HandleInstance,
}

#[derive(Clone)]
pub struct TemplateManager {
    tx: Sender<Template>,
}

impl TemplateManager {
    pub async fn start(&self, template: Template) -> Result<(), SendError<Template>> {
        self.tx.send(template).await
    }
}

pub struct Indexer {
    handlers: Vec<ProcessEventsInput>,
    rx: Receiver<Template>,
    block_handlers: Vec<ProcessBlocksInput>,
    templates: TemplateManager,
    rpc_manager: RPCManager,
}

impl Indexer {
    pub fn new() -> Indexer {
        let (tx, rx) = mpsc::channel::<Template>(1);

        Indexer {
            handlers: Vec::new(),
            block_handlers: Vec::new(),
            templates: TemplateManager { tx },
            rpc_manager: RPCManager::new(),
            rx,
        }
    }

    pub async fn load_event_handler(&mut self, handler: HandleInstance) {
        if handler.is_template() {
            return;
        }

        let provider = self.rpc_manager.get_or_create(handler.network(), handler.rpc_url()).await;

        self.handlers.push(ProcessEventsInput {
            start_block: handler.start_block(),
            address: handler.address(),
            step: 10_000,
            handler,
            templates: self.templates.clone(),
            provider,
        });
    }

    pub async fn load_block_handler(&mut self, handler: BlockHandlerInstance) {
        let provider = self.rpc_manager.get_or_create(handler.network(), handler.rpc_url()).await;

        self.block_handlers.push(ProcessBlocksInput {
            handler,
            templates: self.templates.clone(),
            provider,
        });
    }

    pub async fn start(mut self) {
        for block_handler in self.block_handlers {
            tokio::spawn(async move {
                if let Err(error) = process_logs_block(block_handler).await {
                    println!("Error processing logs for block handler: {error}");
                }
            });
        }

        for handler in self.handlers {
            tokio::spawn(async move {
                if let Err(error) = process_logs(handler).await {
                    println!("Error processing logs for handler: {error}");
                }
            });
        }

        // For dynamic sources (Templates)
        while let Some(template) = self.rx.recv().await {
            let network = template.handler.network();
            let rpc_url = template.handler.rpc_url();
            let provider = self.rpc_manager.get_or_create(network, rpc_url).await;

            let handler = ProcessEventsInput {
                start_block: template.start_block,
                address: template.address,
                step: 10_000,
                handler: template.handler,
                templates: self.templates.clone(),
                provider,
            };

            tokio::spawn(async move {
                if let Err(error) = process_logs(handler).await {
                    println!("Error processing logs for handler: {error}");
                }
            });
        }
    }
}
