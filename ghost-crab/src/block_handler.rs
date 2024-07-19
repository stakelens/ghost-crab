use crate::indexer::TemplateManager;
use crate::latest_block_manager::LatestBlockManager;
use alloy::providers::Provider;
use alloy::providers::RootProvider;
use alloy::rpc::types::eth::Block;
use alloy::rpc::types::eth::BlockNumberOrTag;
use alloy::transports::http::{Client, Http};
use async_trait::async_trait;
use ghost_crab_common::config::ExecutionMode;
use std::sync::Arc;

pub struct BlockContext {
    pub provider: RootProvider<Http<Client>>,
    pub templates: TemplateManager,
    pub block: Block,
}

pub type BlockHandlerInstance = Arc<Box<(dyn BlockHandler + Send + Sync)>>;

#[async_trait]
pub trait BlockHandler {
    async fn handle(&self, params: BlockContext);
    fn get_source(&self) -> String;
}

pub struct BlockConfig {
    pub start_block: u64,
    pub handler: BlockHandlerInstance,
    pub provider: RootProvider<Http<Client>>,
    pub templates: TemplateManager,
    pub step: u64,
    pub execution_mode: ExecutionMode,
}

pub async fn process_logs_block(
    BlockConfig { start_block, handler, provider, templates, step, execution_mode }: BlockConfig,
) {
    let mut current_block = start_block;
    let mut latest_block_manager = LatestBlockManager::new(1000, provider.clone());

    loop {
        let latest_block = latest_block_manager.get().await;

        if current_block >= latest_block {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        let block = provider
            .get_block_by_number(BlockNumberOrTag::Number(current_block), false)
            .await
            .unwrap()
            .unwrap();

        match execution_mode {
            ExecutionMode::Parallel => {
                let handler = handler.clone();
                let provider = provider.clone();
                let templates = templates.clone();

                tokio::spawn(async move {
                    handler.handle(BlockContext { provider, templates, block }).await;
                });
            }
            ExecutionMode::Serial => {
                let templates = templates.clone();
                let provider = provider.clone();
                let templates = templates.clone();

                handler.handle(BlockContext { provider, templates, block }).await;
            }
        }

        current_block += step;
    }
}
