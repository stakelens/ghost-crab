use super::rpc_proxy::RpcWithCache;
use alloy::providers::ProviderBuilder;
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};
use std::collections::HashMap;

pub struct RPCManager {
    current_port: u16,
    rpcs: HashMap<String, RootProvider<Http<Client>>>,
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
