use crate::cache;
use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::eth::{Filter, Log};
use alloy::transports::http::{Client, Http};
use async_trait::async_trait;
use std::sync::Arc;

pub struct Context {
    pub log: Log,
    pub provider: RootProvider<Http<Client>>,
    // pub conn: Arc<Mutex<PgConnection>>,
}

#[async_trait]
pub trait Handler {
    async fn handle(&self, params: Context);
    fn get_source(&self) -> String;
    fn get_event_signature(&self) -> String;
    fn is_template(&self) -> bool;
}

pub struct ProcessLogs {
    pub start_block: u64,
    pub step: u64,
    pub address: String,
    pub handler: Arc<Box<(dyn Handler + Send + Sync)>>,
    pub provider: RootProvider<Http<Client>>,
    // pub conn: Arc<Mutex<PgConnection>>,
}

pub async fn process_log(
    ProcessLogs {
        start_block,
        step,
        address,
        handler,
        provider,
        // conn,
    }: ProcessLogs,
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
                // let conn = Arc::clone(&conn);
                let handler = Arc::clone(&handler);
                let provider = provider.clone();

                tokio::spawn(async move {
                    handler
                        .handle(Context {
                            log,
                            provider,
                            // conn,
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

pub struct DataSourceConfig {
    pub start_block: u64,
    pub step: u64,
    pub address: String,
    pub handler: Arc<Box<(dyn Handler + Send + Sync)>>,
    pub rpc_url: String,
}

pub struct RunInput {
    pub database: String,
    pub data_sources: Vec<DataSourceConfig>,
}

pub async fn run(input: RunInput) {
    let mut rpc_manager = cache::manager::RPCManager::new();
    // let conn = establish_connection(input.database);
    // let conn = Arc::new(Mutex::new(conn));
    let mut processes: Vec<ProcessLogs> = Vec::new();

    for data_source in input.data_sources {
        let process = ProcessLogs {
            start_block: data_source.start_block,
            step: data_source.step,
            address: data_source.address.clone(),
            handler: data_source.handler,
            provider: rpc_manager.get(data_source.rpc_url).await,
            // conn: Arc::clone(&conn),
        };

        processes.push(process);
    }

    let join_handles = processes
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
