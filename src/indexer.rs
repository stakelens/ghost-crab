use std::sync::{Arc, Mutex};

use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::eth::{Filter, Log};
use alloy::transports::http::{Client, Http};
use async_trait::async_trait;
use diesel::PgConnection;

pub struct HandlerParams {
    pub log: Log,
    pub provider: RootProvider<Http<Client>>,
    pub conn: Arc<Mutex<PgConnection>>,
}

#[async_trait]
pub trait Handleable {
    async fn handle(&self, params: HandlerParams);
}

pub struct ProcessLogsParams<'a> {
    pub from_block: u64,
    pub to_block: u64,
    pub address: &'a str,
    pub event: &'a str,
    pub handler: &'a dyn Handleable,
    pub provider: RootProvider<Http<Client>>,
    pub conn: Arc<Mutex<PgConnection>>,
}

pub async fn process_logs_in_range(
    ProcessLogsParams {
        from_block,
        to_block,
        address,
        event,
        handler,
        provider,
        conn,
    }: ProcessLogsParams<'_>,
) {
    let filter = Filter::new()
        .address(address.parse::<Address>().unwrap())
        .event(&event)
        .from_block(from_block)
        .to_block(to_block);

    let logs = provider.get_logs(&filter).await.unwrap();

    for log in logs {
        handler
            .handle(HandlerParams {
                log,
                provider: provider.clone(),
                conn: conn.clone(),
            })
            .await;
    }
}

pub struct ProcessLogs<'a> {
    pub start_block: u64,
    pub step: u64,
    pub event: &'a str,
    pub address: &'a str,
    pub handler: Box<dyn Handleable>,
    pub provider: RootProvider<Http<Client>>,
    pub conn: Arc<Mutex<PgConnection>>,
}

pub async fn process_logs(
    ProcessLogs {
        start_block,
        step,
        address,
        event,
        handler,
        provider,
        conn,
    }: ProcessLogs<'_>,
) {
    let mut current_block = start_block;

    loop {
        let mut end_block = current_block + step;
        let latest_block = provider.get_block_number().await.unwrap();

        if current_block == end_block {
            println!("Reached latest block: {}", current_block);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        if end_block > latest_block {
            end_block = latest_block;
        }

        process_logs_in_range(ProcessLogsParams {
            from_block: current_block,
            to_block: end_block,
            address: address,
            event: event,
            handler: &*handler,
            provider: provider.clone(),
            conn: conn.clone(),
        })
        .await;

        current_block = end_block;
    }
}
