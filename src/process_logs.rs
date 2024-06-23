use crate::handler::{Context, HandlerConfig};
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::eth::Filter;
use std::sync::Arc;

pub async fn process_logs(
    HandlerConfig {
        start_block,
        step,
        address,
        handler,
        provider,
        templates,
    }: HandlerConfig,
) {
    let mut current_block = start_block;
    let handler = Arc::new(handler);
    let event_signature = handler.get_event_signature();
    let address = address.parse::<Address>().unwrap();

    loop {
        let mut end_block = current_block + step;
        let latest_block = provider.get_block_number().await.unwrap();

        if end_block > latest_block {
            end_block = latest_block;
        }

        if current_block >= end_block {
            println!("Reached latest block: {}", current_block);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        println!("Processing logs from {} to {}", current_block, end_block);

        let filter = Filter::new()
            .address(address)
            .event(&event_signature)
            .from_block(current_block)
            .to_block(end_block);

        let logs = provider.get_logs(&filter).await.unwrap();

        let handlers = logs
            .into_iter()
            .map(|log| {
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
                })
            })
            .collect::<Vec<_>>();

        for handle in handlers {
            handle.await.unwrap();
        }

        current_block = end_block;
    }
}
