use crate::indexer::TemplateManager;
use alloy::providers::RootProvider;
use alloy::rpc::types::eth::Log;
use alloy::transports::http::{Client, Http};
use async_trait::async_trait;
use std::sync::Arc;

pub struct Context {
    pub log: Log,
    pub provider: RootProvider<Http<Client>>,
    pub templates: TemplateManager,
}

pub type HandleInstance = Arc<Box<(dyn Handler + Send + Sync)>>;

#[async_trait]
pub trait Handler {
    async fn handle(&self, params: Context);
    fn get_source(&self) -> String;
    fn is_template(&self) -> bool;
    fn get_event_signature(&self) -> String;
}

#[derive(Clone)]
pub struct HandlerConfig {
    pub start_block: u64,
    pub step: u64,
    pub address: String,
    pub handler: HandleInstance,
    pub provider: RootProvider<Http<Client>>,
    pub templates: TemplateManager,
}
