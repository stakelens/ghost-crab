use alloy::providers::ProviderBuilder;
use db::establish_connection;
use indexer::{process_logs, ProcessLogsParams};
use rocketpool::RocketPoolHandler;
mod db;
mod indexer;
mod models;
mod rocketpool;
mod schema;

#[tokio::main]
async fn main() {
    let conn = establish_connection();

    let rpc_url = "".parse().unwrap();

    let provider = ProviderBuilder::new().on_http(rpc_url);

    process_logs(ProcessLogsParams {
        from_block: 19_796_144,
        to_block: 19_796_144 + 10,
        event: "Transfer(address,address,uint256)".parse().unwrap(),
        handler: RocketPoolHandler::new(),
        provider: provider.clone(),
        conn,
    })
    .await;
}
