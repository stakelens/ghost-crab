use crate::indexer::rpc_manager::Provider;
use crate::indexer::templates::TemplateManager;
use crate::latest_block_manager::LatestBlockManager;
use alloy::providers::Provider as AlloyProvider;
use alloy::rpc::types::eth::Block;
use alloy::rpc::types::eth::BlockNumberOrTag;
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
    pub async fn block(&self) -> Result<Option<Block>, TransportError> {
        self.provider.get_block_by_number(BlockNumberOrTag::Number(self.block_number), false).await
    }
}

pub type BlockHandlerInstance = Arc<Box<(dyn BlockHandler + Send + Sync)>>;

#[async_trait]
pub trait BlockHandler {
    async fn handle(&self, params: BlockContext);
    fn name(&self) -> String;
}

#[derive(Clone)]
pub struct ProcessBlocksInput {
    pub handler: BlockHandlerInstance,
    pub templates: TemplateManager,
    pub provider: Provider,
    pub config: BlockHandlerConfig,
}

pub async fn process_blocks(
    ProcessBlocksInput { handler, templates, provider, config }: ProcessBlocksInput,
) -> Result<(), TransportError> {
    let execution_mode = config.execution_mode.unwrap_or(ExecutionMode::Parallel);

    let mut current_block = config.start_block;
    let mut latest_block_manager =
        LatestBlockManager::new(provider.clone(), Duration::from_secs(10));

    loop {
        let latest_block = latest_block_manager.get().await?;

        if current_block >= latest_block {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        match execution_mode {
            ExecutionMode::Parallel => {
                let handler = handler.clone();
                let provider = provider.clone();
                let templates = templates.clone();

                tokio::spawn(async move {
                    handler
                        .handle(BlockContext { provider, templates, block_number: current_block })
                        .await;
                });
            }
            ExecutionMode::Serial => {
                let templates = templates.clone();
                let provider = provider.clone();
                let templates = templates.clone();

                handler
                    .handle(BlockContext { provider, templates, block_number: current_block })
                    .await;
            }
        }

        current_block += config.step;
    }
}
