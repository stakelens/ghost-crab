use crate::indexer::monitoring::HandlerMetrics;
use crate::indexer::progress_manager::ProgressManager;
use crate::indexer::rpc_manager::Provider;
use crate::indexer::templates::TemplateManager;
use crate::latest_block_manager::LatestBlockManager;
use alloy::eips::BlockNumberOrTag;
use alloy::primitives::Address;
use alloy::providers::Provider as AlloyProvider;
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
    pub provider: Provider,
    pub templates: TemplateManager,
    pub contract_address: Address,
    pub is_historical: bool,
}

impl EventContext {
    pub async fn block(&self, hydrate: bool) -> Result<Option<Block>, TransportError> {
        match self.log.block_number {
            Some(block_number) => {
                self.provider
                    .get_block_by_number(BlockNumberOrTag::Number(block_number), hydrate)
                    .await
            }
            None => Err(TransportError::local_usage_str(
                "Error occurred while fetching the current block number within an EventHandler. The log.block_number value is None.",
            )),
        }
    }
}

pub type EventHandlerInstance = Arc<Box<(dyn EventHandler + Send + Sync)>>;

#[async_trait]
pub trait EventHandler {
    async fn handle(
        &self,
        params: EventContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
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
    pub provider: Provider,
    pub execution_mode: ExecutionMode,
    pub metrics: Arc<HandlerMetrics>,
    pub progress: Arc<ProgressManager>,
}

pub async fn process_events(
    ProcessEventsInput {
        handler,
        templates,
        provider,
        metrics,
        progress,
        start_block,
        step,
        address,
        execution_mode,
    }: ProcessEventsInput,
) -> Result<(), TransportError> {
    let event_signature = handler.event_signature();
    let mut current_block = start_block;
    let mut latest_block_manager =
        LatestBlockManager::new(provider.clone(), Duration::from_secs(10));

    loop {
        let mut end_block = current_block + step;
        let latest_block = match latest_block_manager.get().await {
            Ok(block) => block,
            Err(e) => {
                metrics.task_failed(format!("Failed to get latest block: {}", e));
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        let is_historical = latest_block - current_block > 10;

        if end_block > latest_block {
            end_block = latest_block;
        }

        if current_block >= end_block {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        progress.update_progress(&handler.name(), current_block, latest_block).await;

        let filter = Filter::new()
            .address(address)
            .event(&event_signature)
            .from_block(current_block)
            .to_block(end_block);

        let logs = match provider.get_logs(&filter).await {
            Ok(logs) => logs,
            Err(e) => {
                metrics.task_failed(format!("Failed to get logs: {}", e));
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        match execution_mode {
            ExecutionMode::Parallel => {
                for log in logs {
                    let handler = handler.clone();
                    let provider = provider.clone();
                    let templates = templates.clone();
                    let metrics = metrics.clone();

                    metrics.task_started();

                    tokio::spawn(async move {
                        match handler
                            .handle(EventContext {
                                log: log.clone(),
                                provider,
                                templates,
                                contract_address: address,
                                is_historical,
                            })
                            .await
                        {
                            Ok(()) => {
                                let block_number = log.block_number.map(|n| n).unwrap_or(0);
                                metrics.task_completed(block_number)
                            }
                            Err(e) => metrics.task_failed(format!("Handler error: {}", e)),
                        }
                    });
                }
            }
            ExecutionMode::Serial => {
                for log in logs {
                    metrics.task_started();
                    match handler
                        .handle(EventContext {
                            log: log.clone(),
                            provider: provider.clone(),
                            templates: templates.clone(),
                            contract_address: address,
                            is_historical,
                        })
                        .await
                    {
                        Ok(()) => {
                            let block_number = log.block_number.map(|n| n).unwrap_or(0);
                            metrics.task_completed(block_number)
                        }
                        Err(e) => metrics.task_failed(format!("Handler error: {}", e)),
                    }
                }
            }
        }

        current_block = end_block;
    }
}
