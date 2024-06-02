use alloy::providers::{Provider, RootProvider};
use alloy::rpc::types::eth::{Filter, Log};
use alloy::transports::http::{Client, Http};
use diesel::PgConnection;
use async_trait::async_trait;

pub struct HandlerParams<'a> {
    pub log: Log,
    pub provider: RootProvider<Http<Client>>,
    pub conn: &'a mut PgConnection,
}

#[async_trait]
pub trait Handleable {
    async fn handle(&self, params: HandlerParams<'_>);
}

pub struct ProcessLogsParams {
    pub from_block: u64,
    pub to_block: u64,
    pub event: String,
    pub handler: Box<dyn Handleable>,
    pub provider: RootProvider<Http<Client>>,
    pub conn: PgConnection,
}

pub async fn process_logs(
    ProcessLogsParams {
        from_block,
        to_block,
        event,
        handler,
        provider,
        mut conn,
    }: ProcessLogsParams,
) {
    let filter = Filter::new()
        .event(&event)
        .from_block(from_block)
        .to_block(to_block);

    let logs = provider.get_logs(&filter).await.unwrap();

    for log in logs {
        handler.handle(HandlerParams {
            log,
            provider: provider.clone(),
            conn: &mut conn,
        }).await;
    }
}
