use alloy::providers::{Provider, RootProvider};
use alloy::transports::http::{Client, Http};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LatestBlockManager {
    value: u64,
    cache_duration_ms: u128,
    last_fetch_ms: u128,
    provider: RootProvider<Http<Client>>,
}

impl LatestBlockManager {
    pub fn new(cache_duration_ms: u128, provider: RootProvider<Http<Client>>) -> Self {
        Self { value: 0, cache_duration_ms, last_fetch_ms: 0, provider }
    }

    pub async fn get(&mut self) -> u64 {
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        if (now_ms - self.last_fetch_ms) < self.cache_duration_ms {
            return self.value;
        }

        let result = self.provider.get_block_number().await.unwrap();
        self.value = result;

        self.last_fetch_ms = now_ms;

        result
    }
}
