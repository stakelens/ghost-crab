use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Copy, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    Parallel,
    Serial,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub abi: String,
    pub network: String,
    pub execution_mode: Option<ExecutionMode>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSource {
    pub abi: String,
    pub address: String,
    pub start_block: u64,
    pub network: String,
    pub execution_mode: Option<ExecutionMode>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockHandler {
    pub start_block: u64,
    pub network: String,
    pub execution_mode: Option<ExecutionMode>,
    pub step: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub data_sources: HashMap<String, DataSource>,
    pub templates: HashMap<String, Template>,
    pub networks: HashMap<String, String>,
    pub block_handlers: HashMap<String, BlockHandler>,
}
