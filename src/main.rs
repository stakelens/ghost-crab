use indexer::{run, Config, DataSourceConfig};
mod db;
mod handlers;
mod indexer;
mod models;
mod rpc_cache;
mod schema;
use dotenvy::dotenv;
use handlers::rocketpool::RocketPoolHandler;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    run(Config {
        db_url: env::var("DATABASE_URL").unwrap(),
        data_sources: vec![DataSourceConfig {
            start_block: 19_796_144,
            step: 10_000,
            address: "0x6d010c43d4e96d74c422f2e27370af48711b49bf",
            handler: RocketPoolHandler::new(),
            rpc_url: env::var("ETH_RPC_URL").unwrap(),
        }],
    })
    .await;
}
