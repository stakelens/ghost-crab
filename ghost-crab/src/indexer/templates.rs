use alloy::primitives::Address;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;

use crate::event_handler::EventHandlerInstance;

pub struct Template {
    pub start_block: u64,
    pub address: Address,
    pub handler: EventHandlerInstance,
}

#[derive(Clone)]
pub struct TemplateManager {
    tx: Sender<Template>,
}

impl TemplateManager {
    pub fn new(tx: Sender<Template>) -> TemplateManager {
        TemplateManager { tx }
    }

    pub async fn start(&self, template: Template) -> Result<(), SendError<Template>> {
        self.tx.send(template).await
    }
}
