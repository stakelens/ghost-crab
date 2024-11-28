use alloy::transports::TransportError;
use alloy::{eips::BlockNumberOrTag, providers::Provider as AlloyProvider};
use std::time::{Duration, Instant};
use tracing::info;

use crate::indexer::rpc_manager::Provider;

pub struct LatestBlockManager {
    provider: Provider,
    cache_duration: Duration,
    block_number: Option<u64>,
    last_fetch: Instant,
}

impl LatestBlockManager {
    pub fn new(provider: Provider, cache_duration: Duration) -> Self {
        Self { provider, cache_duration, block_number: None, last_fetch: Instant::now() }
    }

    pub async fn get(&mut self) -> Result<u64, TransportError> {
        if let Some(block_number) = self.block_number {
            if self.last_fetch.elapsed() < self.cache_duration {
                return Ok(block_number);
            }
        }

        let latest_finalized_block = self
            .provider
            .get_block_by_number(BlockNumberOrTag::Finalized, false)
            .await?
            .ok_or_else(|| TransportError::local_usage_str("Block not found".into()))?;

        let block_number = latest_finalized_block.header.number.ok_or_else(|| {
            TransportError::local_usage_str("Block number not available in block header".into())
        })?;

        info!("Latest finalized block: {}", block_number);

        self.block_number = Some(block_number);
        self.last_fetch = Instant::now();

        Ok(block_number)
    }
}
