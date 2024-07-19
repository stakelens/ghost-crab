use alloy::providers::{Provider, RootProvider};
use alloy::transports::http::{Client, Http};
use alloy::transports::TransportError;
use std::time::{Duration, Instant};

pub struct LatestBlockManager {
    provider: RootProvider<Http<Client>>,
    cache_duration: Duration,
    block_number: Option<u64>,
    last_fetch: Instant,
}

impl LatestBlockManager {
    pub fn new(provider: RootProvider<Http<Client>>, cache_duration: Duration) -> Self {
        Self { provider, cache_duration, block_number: None, last_fetch: Instant::now() }
    }

    pub async fn get(&mut self) -> Result<u64, TransportError> {
        if let Some(block_number) = self.block_number {
            if self.last_fetch.elapsed() < self.cache_duration {
                return Ok(block_number);
            }
        }

        let block_number = self.provider.get_block_number().await?;

        self.block_number = Some(block_number);
        self.last_fetch = Instant::now();

        Ok(block_number)
    }
}
