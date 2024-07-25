use dotenvy::dotenv;
use serde::Deserialize;
use serde_json::Error as SerdeError;
use std::path::PathBuf;
use std::{collections::HashMap, io::Error as IoError};
use std::{env, fs};

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
pub struct NetworkConfig {
    pub rpc_url: String,
    pub requests_per_second: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub data_sources: HashMap<String, DataSource>,
    pub templates: HashMap<String, Template>,
    pub networks: HashMap<String, NetworkConfig>,
    pub block_handlers: HashMap<String, BlockHandler>,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(IoError),
    CurrentDirNotFound(IoError),
    InvalidConfig(SerdeError),
    EnvVarNotFound(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(error) => {
                write!(formatter, "Config file not found: {}", error)
            }
            ConfigError::CurrentDirNotFound(error) => {
                write!(formatter, "Current directory not found: {}", error)
            }
            ConfigError::InvalidConfig(error) => {
                write!(formatter, "Invalid config format: {}", error)
            }
            ConfigError::EnvVarNotFound(var) => {
                write!(formatter, "Environment variable not found: {}", var)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn load() -> Result<Config, ConfigError> {
    dotenv().ok();

    let config_path = get_config_path()?;
    let config_string = read_config_file(&config_path)?;
    let mut config: Config = parse_config(&config_string)?;
    replace_env_vars(&mut config)?;

    Ok(config)
}

fn get_config_path() -> Result<PathBuf, ConfigError> {
    let current_dir = env::current_dir().map_err(|error| ConfigError::CurrentDirNotFound(error))?;
    Ok(current_dir.join("config.json"))
}

fn read_config_file(path: &PathBuf) -> Result<String, ConfigError> {
    fs::read_to_string(path).map_err(|error| ConfigError::FileNotFound(error))
}

fn parse_config(config_string: &str) -> Result<Config, ConfigError> {
    serde_json::from_str(config_string).map_err(|error| ConfigError::InvalidConfig(error))
}

fn replace_env_vars(config: &mut Config) -> Result<(), ConfigError> {
    for (_key, value) in &mut config.networks {
        if value.rpc_url.starts_with('$') {
            (*value).rpc_url = env::var(&value.rpc_url[1..])
                .map_err(|_| ConfigError::EnvVarNotFound(value.rpc_url[1..].to_string()))?;
        }
    }

    Ok(())
}
