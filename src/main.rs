use alloy::providers::ProviderBuilder;
use db::establish_connection;
use indexer::{process_logs, ProcessLogs};
use rocketpool::RocketPoolHandler;
use std::sync::{Arc, Mutex};
mod db;
mod indexer;
mod models;
mod rocketpool;
mod schema;

#[tokio::main]
async fn main() {
    let conn = establish_connection();

    let rpc_url = ""
        .parse()
        .unwrap();

    let provider = ProviderBuilder::new().on_http(rpc_url);

    process_logs(ProcessLogs {
        start_block: 19_796_144,
        step: 10_000,
        address: "0x6d010c43d4e96d74c422f2e27370af48711b49bf",
        event: "MinipoolCreated(address,address,uint256)",
        handler: &*RocketPoolHandler::new(),
        provider: provider.clone(),
        conn: Arc::new(Mutex::new(conn)),
    })
    .await;
}
