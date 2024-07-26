use super::cache::load_cache;
use super::error::{Error, Result};
use crate::layers::cache_layer::CacheLayer;
use crate::layers::cache_layer::CacheService;
use crate::layers::rate_limit_layer::RateLimit;
use crate::layers::rate_limit_layer::RateLimitLayer;
use alloy::providers::ProviderBuilder;
use alloy::providers::RootProvider;
use alloy::rpc::client::ClientBuilder;
use alloy::transports::http::reqwest::Url;
use alloy::transports::http::{Client, Http};
use std::collections::HashMap;
use std::time::Duration;

pub type Provider = RootProvider<CacheService<RateLimit<Http<Client>>>>;

pub struct RPCManager {
    rpcs: HashMap<String, Provider>,
}

impl RPCManager {
    pub fn new() -> Self {
        RPCManager { rpcs: HashMap::new() }
    }

    pub async fn get_or_create(
        &mut self,
        network: String,
        rpc_url: String,
        rate_limit: u64,
    ) -> Result<Provider> {
        if let Some(provider) = self.rpcs.get(&rpc_url) {
            return Ok(provider.clone());
        }

        let url = Url::parse(&rpc_url).map_err(|e| Error::InvalidRpcUrl(Box::new(e)))?;
        let cache = load_cache(&network)?;

        let cache_layer = CacheLayer::new(cache);
        let rate_limit_layer = RateLimitLayer::new(rate_limit, Duration::from_secs(1));

        let client = ClientBuilder::default().layer(cache_layer).layer(rate_limit_layer).http(url);
        let provider = ProviderBuilder::new().on_client(client);

        self.rpcs.insert(rpc_url.clone(), provider.clone());

        Ok(provider)
    }
}
