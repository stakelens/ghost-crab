use super::rpc_proxy::RpcWithCache;
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
}

impl Default for RPCManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RPCManager {
    pub fn new() -> Self {
        RPCManager { rpcs: HashMap::new(), current_port: 3001 }
    }

    pub async fn get_or_create(
        &mut self,
        network: String,
        rpc_url: String,
    ) -> RootProvider<Http<Client>> {
        let provider = self.rpcs.get(&rpc_url);

        match provider {
            Some(value) => value.clone(),
            None => {
                let provider = ProviderBuilder::new()
                    .on_http(format!("http://localhost:{}", self.current_port).parse().unwrap());

                self.rpcs.insert(rpc_url.clone(), provider.clone());
                let rpc_with_cache =
                    RpcWithCache::new(network, rpc_url.clone(), self.current_port).unwrap();

                tokio::spawn(async move {
                    rpc_with_cache.run().await.unwrap();
                });

                self.current_port += 1;
                provider
            }
        }
    }
}
