use crate::handler::{Context, HandlerConfig};
use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::eth::Filter;
use alloy::transports::http::{Client, Http};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

struct LatestBlockManager {
    value: u64,
    cache_duration_ms: u128,
    last_fetch_ms: u128,
    provider: RootProvider<Http<Client>>,
}

impl LatestBlockManager {
    fn new(cache_duration_ms: u128, provider: RootProvider<Http<Client>>) -> Self {
        return Self {
            value: 0,
            cache_duration_ms,
            last_fetch_ms: 0,
            provider,
        };
    }

    async fn get(&mut self) -> u64 {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        if (now_ms - self.last_fetch_ms) < self.cache_duration_ms {
            return self.value;
        }

        let result = self.provider.get_block_number().await.unwrap();
        self.value = result;

        self.last_fetch_ms = now_ms;

        return result;
    }
}

#[derive(Clone, Copy, Deserialize, Debug)]
#[serde(rename_all="lowercase")]
pub enum ExecutionMode {
    Parallel,
    Serial,
}

pub async fn process_logs(
    HandlerConfig {
        start_block,
        step,
        address,
        handler,
        provider,
        templates,
        execution_mode,
    }: HandlerConfig,
) {
    let mut current_block = start_block;
    let event_signature = handler.get_event_signature();
    let address = address.parse::<Address>().unwrap();

    let mut block_manager = LatestBlockManager::new(1000, provider.clone());

    loop {
        let mut end_block = current_block + step;
        let latest_block = block_manager.get().await;

        if end_block > latest_block {
            end_block = latest_block;
        }

        if current_block >= end_block {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        let source = handler.get_source();

        println!(
            "[{}] Processing logs from {} to {}",
            source, current_block, end_block
        );

        let filter = Filter::new()
            .address(address)
            .event(&event_signature)
            .from_block(current_block)
            .to_block(end_block);

        let logs = provider.get_logs(&filter).await.unwrap();

        match execution_mode {
            ExecutionMode::Parallel => {
                for log in logs {
                    let handler = handler.clone();
                    let provider = provider.clone();
                    let templates = templates.clone();

                    tokio::spawn(async move {
                        handler
                            .handle(Context {
                                log,
                                provider,
                                templates,
                            })
                            .await;
                    });
                }
            }
            ExecutionMode::Serial => {
                for log in logs {
                    let templates = templates.clone();
                    let provider = provider.clone();
                    let templates = templates.clone();

                    handler
                        .handle(Context {
                            log,
                            provider,
                            templates,
                        })
                        .await;
                }
            }
        }

        current_block = end_block;
    }
}
