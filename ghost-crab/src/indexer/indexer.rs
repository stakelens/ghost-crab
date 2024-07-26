use crate::block_handler::{process_blocks, BlockHandlerInstance, ProcessBlocksInput};
use crate::cache::manager::{CacheProvider, RPCManager};
use crate::event_handler::{process_events, EventHandlerInstance, ProcessEventsInput};

use alloy::primitives::Address;
use ghost_crab_common::config::{self, Config, ConfigError};
use tokio::sync::mpsc::{self, Receiver};

use super::error::AddHandlerError;
use super::templates::{Template, TemplateManager};

pub struct Indexer {
    handlers: Vec<ProcessEventsInput>,
    rx: Receiver<Template>,
    block_handlers: Vec<ProcessBlocksInput>,
    templates: TemplateManager,
    rpc_manager: RPCManager,
    config: Config,
}

impl Indexer {
    pub fn new() -> Result<Indexer, ConfigError> {
        let (tx, rx) = mpsc::channel::<Template>(1);

        let config = config::load()?;

        Ok(Indexer {
            config,
            handlers: Vec::new(),
            block_handlers: Vec::new(),
            templates: TemplateManager::new(tx),
            rpc_manager: RPCManager::new(),
            rx,
        })
    }

    pub async fn load_event_handler(
        &mut self,
        handler: EventHandlerInstance,
    ) -> Result<(), AddHandlerError> {
        let event_config = self
            .config
            .data_sources
            .remove(&handler.name())
            .ok_or(AddHandlerError::NotFound(handler.name()))?;

        let provider = self.get_provider(&event_config.network).await?;

        let address = str::parse::<Address>(&event_config.address).map_err(|error| {
            AddHandlerError::InvalidAddress { address: event_config.address, error }
        })?;

        self.handlers.push(ProcessEventsInput {
            start_block: event_config.start_block,
            address,
            step: 10_000,
            handler,
            templates: self.templates.clone(),
            provider,
            execution_mode: event_config.execution_mode.unwrap_or(config::ExecutionMode::Parallel),
        });

        Ok(())
    }

    pub async fn load_block_handler(
        &mut self,
        handler: BlockHandlerInstance,
    ) -> Result<(), AddHandlerError> {
        let block_config = self
            .config
            .block_handlers
            .remove(&handler.name())
            .ok_or(AddHandlerError::NotFound(handler.name()))?;

        let provider = self.get_provider(&block_config.network).await?;

        self.block_handlers.push(ProcessBlocksInput {
            handler,
            templates: self.templates.clone(),
            provider,
            config: block_config,
        });

        Ok(())
    }

    async fn get_provider(&mut self, network_name: &str) -> Result<CacheProvider, AddHandlerError> {
        let network = self
            .config
            .networks
            .get(network_name)
            .ok_or(AddHandlerError::NetworkNotFound(network_name.to_string()))?;

        let provider = self
            .rpc_manager
            .get_or_create(
                network_name.to_string(),
                network.rpc_url.clone(),
                network.requests_per_second,
            )
            .await;

        Ok(provider)
    }

    pub async fn start(mut self) -> Result<(), AddHandlerError> {
        for block_handler in self.block_handlers.clone() {
            tokio::spawn(async move {
                if let Err(error) = process_blocks(block_handler).await {
                    println!("Error processing logs for block handler: {error}");
                }
            });
        }

        for handler in self.handlers.clone() {
            tokio::spawn(async move {
                if let Err(error) = process_events(handler).await {
                    println!("Error processing logs for handler: {error}");
                }
            });
        }

        // For dynamic sources (Templates)
        while let Some(template) = self.rx.recv().await {
            let config = self
                .config
                .templates
                .get(&template.handler.name())
                .ok_or(AddHandlerError::NotFound(template.handler.name()))?;

            let execution_mode = config.execution_mode.unwrap_or(config::ExecutionMode::Parallel);
            let provider = self.get_provider(&config.network.clone()).await?;

            let handler = ProcessEventsInput {
                start_block: template.start_block,
                address: template.address,
                step: 10_000,
                handler: template.handler,
                templates: self.templates.clone(),
                provider,
                execution_mode,
            };

            tokio::spawn(async move {
                if let Err(error) = process_events(handler).await {
                    println!("Error processing logs for handler: {error}");
                }
            });
        }

        Ok(())
    }
}
