use crate::indexer::monitoring::HandlerMetrics;
use crate::indexer::progress_manager::ProgressManager;
use crate::indexer::rpc_manager::Provider;
use crate::indexer::templates::TemplateManager;
use crate::latest_block_manager::LatestBlockManager;
use alloy::providers::Provider as AlloyProvider;
use alloy::rpc::types::eth::BlockNumberOrTag;
use alloy::rpc::types::Block;
use alloy::transports::TransportError;
use async_trait::async_trait;
use ghost_crab_common::config::BlockHandlerConfig;
use ghost_crab_common::config::ExecutionMode;
use std::sync::Arc;
use std::time::Duration;

pub struct BlockContext {
    pub provider: Provider,
    pub templates: TemplateManager,
    pub block_number: u64,
}

impl BlockContext {
    pub async fn block(&self, hydrate: bool) -> Result<Option<Block>, TransportError> {
        self.provider
            .get_block_by_number(BlockNumberOrTag::Number(self.block_number), hydrate)
            .await
    }
}

pub type BlockHandlerInstance = Arc<Box<(dyn BlockHandler + Send + Sync)>>;

#[async_trait]
pub trait BlockHandler {
    async fn handle(
        &self,
        params: BlockContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn name(&self) -> String;
}

#[derive(Clone)]
pub struct ProcessBlocksInput {
    pub handler: BlockHandlerInstance,
    pub templates: TemplateManager,
    pub provider: Provider,
    pub config: BlockHandlerConfig,
    pub metrics: Arc<HandlerMetrics>,
    pub progress: Arc<ProgressManager>,
}

pub async fn process_blocks(
    ProcessBlocksInput {
        handler,
        templates,
        provider,
        config,
        metrics,
        progress,
    }: ProcessBlocksInput,
) -> Result<(), TransportError> {
    let execution_mode = config.execution_mode.unwrap_or(ExecutionMode::Parallel);
    let mut current_block = config.start_block;
    let mut latest_block_manager =
        LatestBlockManager::new(provider.clone(), Duration::from_secs(10));

    loop {
        let latest_block = match latest_block_manager.get().await {
            Ok(block) => block,
            Err(e) => {
                metrics.task_failed(format!("Failed to get latest block: {}", e));
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        if current_block >= latest_block {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        progress.update_progress(&handler.name(), current_block, latest_block).await;

        metrics.task_started();

        match execution_mode {
            ExecutionMode::Parallel => {
                let handler = handler.clone();
                let provider = provider.clone();
                let templates = templates.clone();
                let metrics = metrics.clone();

                tokio::spawn(async move {
                    match handler
                        .handle(BlockContext { provider, templates, block_number: current_block })
                        .await
                    {
                        Ok(()) => metrics.task_completed(current_block),
                        Err(e) => metrics.task_failed(format!("Handler error: {}", e)),
                    }
                });
            }
            ExecutionMode::Serial => {
                match handler
                    .handle(BlockContext {
                        provider: provider.clone(),
                        templates: templates.clone(),
                        block_number: current_block,
                    })
                    .await
                {
                    Ok(()) => metrics.task_completed(current_block),
                    Err(e) => metrics.task_failed(format!("Handler error: {}", e)),
                }
            }
        }

        current_block += config.step;
    }
}
