use alloy::providers::ProviderBuilder;
use db::establish_connection;
use indexer::{process_log, ProcessLogs};
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

    let rpc_url = "".parse().unwrap();

    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_url));
    let conn = Arc::new(Mutex::new(conn));

    let indexer_processes: Vec<ProcessLogs> = vec![ProcessLogs {
        start_block: 19_796_144,
        step: 10_000,
        address: "0x6d010c43d4e96d74c422f2e27370af48711b49bf",
        event: "MinipoolCreated(address,address,uint256)",
        handler: RocketPoolHandler::new(),
        provider: Arc::clone(&provider),
        conn: Arc::clone(&conn),
    }];

    let join_handles = indexer_processes
        .into_iter()
        .map(|process| {
            tokio::spawn(async move {
                process_log(process).await;
            })
        })
        .collect::<Vec<_>>();

    for handle in join_handles {
        handle.await.unwrap();
    }
}
