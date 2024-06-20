use crate::cache;
use crate::db::establish_connection;
use alloy::primitives::Address;
use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::eth::{Filter, Log};
use alloy::transports::http::{Client, Http};
use async_trait::async_trait;
use diesel::PgConnection;
use std::sync::{Arc, Mutex};

pub struct Context {
    pub log: Log,
    pub provider: RootProvider<Http<Client>>,
    pub conn: Arc<Mutex<PgConnection>>,
}

#[async_trait]
pub trait Handleable {
    async fn handle(&self, params: Context);
    fn get_data_source(&self) -> String;
    fn get_event_signature(&self) -> String;
}

pub struct ProcessLogsParams<'a> {
    pub filter: Filter,
    pub handler: Arc<Box<(dyn Handleable + Send + Sync)>>,
    pub provider: &'a RootProvider<Http<Client>>,
    pub conn: Arc<Mutex<PgConnection>>,
}

pub async fn process_logs_in_range(
    ProcessLogsParams {
        filter,
        handler,
        provider,
        conn,
    }: ProcessLogsParams<'_>,
) {
    let logs = provider.get_logs(&filter).await.unwrap();

    let handlers = logs
        .into_iter()
        .map(|log| {
            let conn = Arc::clone(&conn);
            let handler = Arc::clone(&handler);
            let provider = provider.clone();

            tokio::spawn(async move {
                handler
                    .handle(Context {
                        log,
                        provider,
                        conn,
                    })
                    .await;
            })
        })
        .collect::<Vec<_>>();

    for handle in handlers {
        handle.await.unwrap();
    }
}

pub struct ProcessLogs {
    pub start_block: u64,
    pub step: u64,
    pub address: String,
    pub handler: Box<(dyn Handleable + Send + Sync)>,
    pub provider: Arc<RootProvider<Http<Client>>>,
    pub conn: Arc<Mutex<PgConnection>>,
}

pub async fn process_log(
    ProcessLogs {
        start_block,
        step,
        address,
        handler,
        provider,
        conn,
    }: ProcessLogs,
) {
    let mut current_block = start_block;
    let handler = Arc::new(handler);
    let event_signature = handler.get_event_signature();
    let address = address.parse::<Address>().unwrap();

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

        println!("Processing logs from {} to {}", current_block, end_block);

        let filter = Filter::new()
            .address(address)
            .event(&event_signature)
            .from_block(current_block)
            .to_block(end_block);

        process_logs_in_range(ProcessLogsParams {
            filter,
            handler: Arc::clone(&handler),
            provider: &provider,
            conn: conn.clone(),
        })
        .await;

        current_block = end_block;
    }
}

pub struct DataSourceConfig {
    pub start_block: u64,
    pub step: u64,
    pub address: String,
    pub handler: Box<(dyn Handleable + Send + Sync)>,
    pub rpc_url: String,
}

pub struct Config {
    pub db_url: String,
    pub data_sources: Vec<DataSourceConfig>,
}

pub async fn run(config: Config) {
    let mut ingesters = cache::manager::RPCManager::new(config.db_url.clone());
    let conn = establish_connection(config.db_url);

    let conn = Arc::new(Mutex::new(conn));

    let handlers = config
        .data_sources
        .into_iter()
        .map(|data_source| ProcessLogs {
            start_block: data_source.start_block,
            step: data_source.step,
            address: data_source.address.clone(),
            handler: data_source.handler,
            provider: Arc::new(ingesters.get(data_source.rpc_url)),
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
