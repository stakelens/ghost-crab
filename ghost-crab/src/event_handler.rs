use crate::cache::manager::CacheProvider;
use crate::indexer::templates::TemplateManager;
use crate::latest_block_manager::LatestBlockManager;
use alloy::eips::BlockNumberOrTag;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::eth::Filter;
use alloy::rpc::types::eth::Log;
use alloy::rpc::types::Block;
use alloy::transports::TransportError;
use async_trait::async_trait;
use ghost_crab_common::config::ExecutionMode;
use std::sync::Arc;
use std::time::Duration;

pub struct EventContext {
    pub log: Log,
    pub provider: CacheProvider,
    pub templates: TemplateManager,
    pub contract_address: Address,
}

impl EventContext {
    pub async fn block(&self) -> Result<Option<Block>, TransportError> {
        match self.log.block_number {
            Some(block_number) => {
                self.provider
                    .get_block_by_number(BlockNumberOrTag::Number(block_number), false)
                    .await
            }
            None => Err(TransportError::local_usage_str("Error occurred while fetching the current block number within an EventHandler. The log.block_number value is None.")),
        }
    }
}

pub type EventHandlerInstance = Arc<Box<(dyn EventHandler + Send + Sync)>>;

#[async_trait]
pub trait EventHandler {
    async fn handle(&self, params: EventContext);
    fn name(&self) -> String;
    fn event_signature(&self) -> String;
}

#[derive(Clone)]
pub struct ProcessEventsInput {
    pub start_block: u64,
    pub address: Address,
    pub step: u64,
    pub handler: EventHandlerInstance,
    pub templates: TemplateManager,
    pub provider: CacheProvider,
    pub execution_mode: ExecutionMode,
}

pub async fn process_events(
    ProcessEventsInput { start_block, execution_mode, step, address, handler, templates, provider }: ProcessEventsInput,
) -> Result<(), TransportError> {
    let event_signature = handler.event_signature();

    let mut current_block = start_block;
    let mut latest_block_manager =
        LatestBlockManager::new(provider.clone(), Duration::from_secs(10));

    loop {
        let mut end_block = current_block + step;
        let latest_block = latest_block_manager.get().await?;

        if end_block > latest_block {
            end_block = latest_block;
        }

        if current_block >= end_block {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        let source = handler.name();

        println!("[{}] Processing logs from {} to {}", source, current_block, end_block);

        let filter = Filter::new()
            .address(address)
            .event(&event_signature)
            .from_block(current_block)
            .to_block(end_block);

        let logs = provider.get_logs(&filter).await?;

        match execution_mode {
            ExecutionMode::Parallel => {
                for log in logs {
                    let handler = handler.clone();
                    let provider = provider.clone();
                    let templates = templates.clone();

                    tokio::spawn(async move {
                        handler
                            .handle(EventContext {
                                log,
                                provider,
                                templates,
                                contract_address: address,
                            })
                            .await;
                    });
                }
            }
            ExecutionMode::Serial => {
                for log in logs {
                    let templates = templates.clone();
                    let provider = provider.clone();

                    handler
                        .handle(EventContext {
                            log,
                            provider,
                            templates,
                            contract_address: address,
                        })
                        .await;
                }
            }
        }

        current_block = end_block;
    }
}
