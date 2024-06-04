use alloy::providers::ProviderBuilder;
use db::establish_connection;
use indexer::{process_log, ProcessLogs, ProcessLogsConfig};
use rocketpool::RocketPoolHandler;
use std::sync::{Arc, Mutex};
mod db;
mod indexer;
mod models;
mod rocketpool;
mod schema;
use dotenvy::dotenv;
use std::env;

struct Config<'a> {
    rpc_url: String,
    db_url: String,
    handlers: Vec<ProcessLogsConfig<'a>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    run(Config {
        rpc_url: env::var("RPC_URL").unwrap(),
        db_url: env::var("DATABASE_URL").unwrap(),
        handlers: vec![ProcessLogsConfig {
            start_block: 19_796_144,
            step: 10_000,
            address: "0x6d010c43d4e96d74c422f2e27370af48711b49bf",
            handler: RocketPoolHandler::new(),
        }],
    })
    .await;
}

async fn run(config: Config<'static>) {
    let conn = establish_connection(config.db_url);
    let rpc_url = config.rpc_url.parse().unwrap();
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_url));
    let conn = Arc::new(Mutex::new(conn));

    let handlers = config
        .handlers
        .into_iter()
        .map(|config| ProcessLogs {
            start_block: config.start_block,
            step: config.step,
            address: config.address,
            handler: RocketPoolHandler::new(),
            provider: Arc::clone(&provider),
            conn: Arc::clone(&conn),
        })
        .collect::<Vec<_>>();

    let join_handles = handlers
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
