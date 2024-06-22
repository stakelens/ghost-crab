use super::rpc_proxy::RpcWithCache;
use crate::config;
use alloy::providers::ProviderBuilder;
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub static RPC_MANAGER: Lazy<Arc<Mutex<RPCManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(RPCManager::new())));

pub struct RPCManager {
    current_port: u16,
    rpcs: HashMap<String, RootProvider<Http<Client>>>,
    config: config::Config,
}

impl RPCManager {
    pub fn new() -> Self {
        RPCManager {
            rpcs: HashMap::new(),
            current_port: 3000,
            config: config::load(),
        }
    }

    pub async fn get(&mut self, network: String) -> RootProvider<Http<Client>> {
        let rpc_url = self.config.networks.get(&network).unwrap();
        let provider = self.rpcs.get(rpc_url);

        match provider {
            Some(value) => {
                return value.clone();
            }
            None => {
                let provider = ProviderBuilder::new().on_http(
                    format!("http://localhost:{}", self.current_port)
                        .parse()
                        .unwrap(),
                );

                self.rpcs.insert(rpc_url.clone(), provider.clone());
                let rpc_with_cache = RpcWithCache::new(network, rpc_url.clone(), self.current_port);

                tokio::spawn(async move {
                    rpc_with_cache.run().await;
                });

                self.current_port = self.current_port + 1;
                return provider;
            }
        }
    }
}
