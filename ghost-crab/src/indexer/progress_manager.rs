use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct ProgressManager {
    progress: RwLock<HashMap<String, u64>>,
}

impl ProgressManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { progress: RwLock::new(HashMap::new()) })
    }

    pub async fn update_progress(&self, handler_name: &str, current_block: u64, target_block: u64) {
        let mut progress = self.progress.write().await;
        progress.insert(handler_name.to_string(), current_block);

        info!("Handler '{}' at block {} (target: {})", handler_name, current_block, target_block,);
    }
}
