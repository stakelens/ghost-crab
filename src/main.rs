use alloy::providers::ProviderBuilder;
use db::establish_connection;
use indexer::{process_log, DataSourceConfig, ProcessLogs};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
mod db;
mod handlers;
mod indexer;
mod models;
mod rpc_cache;
mod schema;
use dotenvy::dotenv;
use handlers::rocketpool::RocketPoolHandler;
use std::env;

struct RpcConfig {
    rpc_urls: HashMap<u64, String>,
}

struct Config<'a> {
    rpc_config: RpcConfig,
    db_url: String,
    data_sources: Vec<DataSourceConfig<'a>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    run(Config {
        rpc_config: RpcConfig {
            rpc_urls: HashMap::from([
                (1, env::var("ETH_RPC_URL").unwrap()),
                (10, env::var("OPT_RPC_URL").unwrap()),
            ]),
        },
        db_url: env::var("DATABASE_URL").unwrap(),
        data_sources: vec![DataSourceConfig {
            start_block: 19_796_144,
            step: 10_000,
            address: "0x6d010c43d4e96d74c422f2e27370af48711b49bf",
            handler: RocketPoolHandler::new(),
            ingester: Arc::new(
                ProviderBuilder::new().on_http("http://localhost:3000".parse().unwrap()),
            ),
        }],
    })
    .await;
}

async fn start_cache(config: &Config<'static>) {
    let eth_rpc_with_cache = rpc_cache::RpcWithCache::new(
        config.db_url.clone(),
        config.rpc_config.rpc_urls.get(&1).unwrap().clone(),
        3000,
        1, // Ethereum Mainnet (ChainID: 1)
    );

    tokio::spawn(async move {
        eth_rpc_with_cache.run().await;
    });

    // Optimism (ChainID: 10)
    let opt_rpc_with_cache = rpc_cache::RpcWithCache::new(
        config.db_url.clone(),
        config.rpc_config.rpc_urls.get(&10).unwrap().clone(),
        3001,
        10, // Optimism (ChainID: 10)
    );

    tokio::spawn(async move {
        opt_rpc_with_cache.run().await;
    });
}

async fn run(config: Config<'static>) {
    start_cache(&config).await;

    let conn = establish_connection(config.db_url);

    let conn = Arc::new(Mutex::new(conn));

    let handlers = config
        .data_sources
        .into_iter()
        .map(|config| ProcessLogs {
            start_block: config.start_block,
            step: config.step,
            address: config.address,
            handler: config.handler,
            provider: config.ingester,
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
