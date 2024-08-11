use super::server;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};

#[derive(Serialize, Clone)]
pub struct ProgressState {
    pub label: String,
    pub start_block: u64,
    pub current_block: u64,
    pub end_block: u64,
}

pub struct ProgressUpdate {
    id: usize,
    payload: ProgressUpdatePayload,
}

pub enum ProgressUpdatePayload {
    UpdateCurrentBlock(u64),
    UpdateEndBlock(u64),
}

pub struct ProgressManager {
    progress_states: Arc<Mutex<Vec<ProgressState>>>,
    indexer_receiver: Option<Receiver<ProgressUpdate>>,
    indexer_sender: Sender<ProgressUpdate>,
}

impl ProgressManager {
    pub fn new() -> ProgressManager {
        let (indexer_sender, indexer_receiver) = mpsc::channel::<ProgressUpdate>(10_000);

        let progress_states = Arc::new(Mutex::new(Vec::new()));
        let server_progress_states = Arc::clone(&progress_states);

        tokio::spawn(async move {
            server::init_server(server_progress_states).await;
        });

        ProgressManager {
            progress_states,
            indexer_sender,
            indexer_receiver: Some(indexer_receiver),
        }
    }

    pub async fn create_progress(&mut self, label: String, start_block: u64) -> ProgressChannel {
        let progress_state = ProgressState { label, start_block, current_block: 0, end_block: 0 };
        let id = self.progress_states.lock().await.len();
        self.progress_states.lock().await.push(progress_state);
        ProgressChannel::new(id, self.indexer_sender.clone())
    }

    pub fn start(&mut self) {
        let mut indexer_receiver =
            self.indexer_receiver.take().expect("Indexer receiver is missing");

        let progress_states = Arc::clone(&self.progress_states);

        tokio::spawn(async move {
            while let Some(message) = indexer_receiver.recv().await {
                let progress_states = Arc::clone(&progress_states);
                let mut progress_states = progress_states.lock().await;

                match message.payload {
                    ProgressUpdatePayload::UpdateCurrentBlock(current_block) => {
                        if current_block > progress_states[message.id].current_block {
                            progress_states[message.id].current_block = current_block;
                        }
                    }
                    ProgressUpdatePayload::UpdateEndBlock(end_block) => {
                        progress_states[message.id].end_block = end_block;
                    }
                }
            }
        });
    }
}

#[derive(Clone)]
pub struct ProgressChannel {
    id: usize,
    channel: Sender<ProgressUpdate>,
}

impl ProgressChannel {
    pub fn new(id: usize, channel: Sender<ProgressUpdate>) -> ProgressChannel {
        ProgressChannel { id, channel }
    }

    pub async fn send(&self, payload: ProgressUpdatePayload) {
        self.channel
            .send(ProgressUpdate { id: self.id, payload })
            .await
            .expect("Failed to send progress update");
    }
}
