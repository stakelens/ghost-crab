use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::eth::{Filter, Log};
use alloy::transports::http::{Client, Http};

pub struct HandlerParams {
    pub log: Log,
    pub client: RootProvider<Http<Client>>,
}

type Handler = fn(HandlerParams);

pub struct ProcessLogsParams {
    pub from_block: u64,
    pub to_block: u64,
    pub event: String,
    pub handler: Handler,
    pub provider: RootProvider<Http<Client>>,
}

pub async fn process_logs(
    ProcessLogsParams {
        from_block,
        to_block,
        event,
        handler,
        provider,
    }: ProcessLogsParams,
) {
    let filter = Filter::new()
        .event(&event)
        .from_block(from_block)
        .to_block(to_block);

    let logs = provider.get_logs(&filter).await.unwrap();

    for log in logs {
        handler(HandlerParams {
            log,
            client: provider.clone(),
        });
    }
}
