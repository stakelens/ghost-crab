use alloy::providers::ProviderBuilder;
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};
use once_cell::sync::Lazy;
use rocksdb::DB;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::rpc_proxy::RpcWithCache;

pub static RPC_MANAGER: Lazy<Arc<Mutex<RPCManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(RPCManager::new())));

pub struct RPCManager {
    current_port: u16,
    cache: Arc<DB>,
    rpcs: HashMap<String, RootProvider<Http<Client>>>,
}

impl RPCManager {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap();
        let cache = Arc::new(DB::open_default(current_dir.join("cache")).unwrap());

        RPCManager {
            rpcs: HashMap::new(),
            cache,
            current_port: 3000,
        }
    }

    pub async fn get(&mut self, rpc_url: String) -> RootProvider<Http<Client>> {
        let result = self.rpcs.get(&rpc_url);

        match result {
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

                let rpc_with_cache =
                    RpcWithCache::new(Arc::clone(&self.cache), rpc_url.clone(), self.current_port);

                tokio::spawn(async move {
                    rpc_with_cache.run().await;
                });

                self.current_port = self.current_port + 1;
                return provider;
            }
        }
    }
}
