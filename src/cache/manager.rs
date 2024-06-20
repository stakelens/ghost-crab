use alloy::providers::ProviderBuilder;
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};
use std::collections::HashMap;

use super::rpc_proxy::RpcWithCache;

pub struct RPCManager {
    db_url: String,
    current_port: u16,
    rpcs: HashMap<String, RootProvider<Http<Client>>>,
}

impl RPCManager {
    pub fn new(db_url: String) -> Self {
        RPCManager {
            db_url: db_url,
            current_port: 3000,
            rpcs: HashMap::new(),
        }
    }

    pub fn get(&mut self, rpc_url: String) -> RootProvider<Http<Client>> {
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

                // Start the Ingester service
                let rpc_with_cache =
                    RpcWithCache::new(self.db_url.clone(), rpc_url.clone(), self.current_port);

                tokio::spawn(async move {
                    rpc_with_cache.run().await;
                });

                self.current_port = self.current_port + 1;
                return provider;
            }
        }
    }
}
