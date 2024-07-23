use alloy::providers::ProviderBuilder;
use alloy::providers::RootProvider;
use alloy::rpc::client::ClientBuilder;
use alloy::transports::http::{Client, Http};
use std::collections::HashMap;

use super::cache::load_cache;
use super::cache_layer::CacheLayer;
use super::cache_layer::CacheService;

pub type CacheProvider = RootProvider<CacheService<Http<Client>>>;

pub struct RPCManager {
    rpcs: HashMap<String, CacheProvider>,
}

impl RPCManager {
    pub fn new() -> Self {
        RPCManager { rpcs: HashMap::new() }
    }

    pub async fn get_or_create(&mut self, network: String, rpc_url: String) -> CacheProvider {
        if let Some(provider) = self.rpcs.get(&rpc_url) {
            return provider.clone();
        }

        let cache = load_cache(&network).unwrap();
        let cache_layer = CacheLayer::new(cache);

        let client = ClientBuilder::default().layer(cache_layer).http(rpc_url.parse().unwrap());
        let provider = ProviderBuilder::new().on_client(client);

        self.rpcs.insert(rpc_url.clone(), provider.clone());

        provider
    }
}
