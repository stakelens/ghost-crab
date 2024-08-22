use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use super::prometheus::Prometheus;

#[derive(Clone, Debug)]
pub enum ProgressUpdatePayload {
    IncrementProcessedBlocks(u64),
    UpdateEndBlock(u64),
    SetStartBlock(u64),
}

pub struct ProgressState {
    pub label: String,
    pub start_block: u64,
    pub current_block: u64,
    pub end_block: u64,
}

pub struct ProgressManager {
    state: Arc<Mutex<Vec<ProgressState>>>,
    sender: broadcast::Sender<(usize, ProgressUpdatePayload)>,
    receiver: Option<broadcast::Receiver<(usize, ProgressUpdatePayload)>>,
    prometheus: Option<Prometheus>,
}

impl ProgressManager {
    pub fn new() -> Self {
        let (sender, receiver) = broadcast::channel::<(usize, ProgressUpdatePayload)>(10_000);
        let state = Arc::new(Mutex::new(Vec::new()));
        ProgressManager { state, sender, receiver: Some(receiver), prometheus: None }
    }

    pub fn start(&mut self) {
        let mut receiver = self.receiver.take().expect("ProccessManager already started");
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            loop {
                if let Ok((id, payload)) = receiver.recv().await {
                    let state = Arc::clone(&state);
                    let mut state = state.lock().await;

                    match payload {
                        ProgressUpdatePayload::IncrementProcessedBlocks(amount) => {
                            state[id].current_block += amount;
                        }
                        ProgressUpdatePayload::UpdateEndBlock(end_block) => {
                            state[id].end_block = end_block;
                        }
                        ProgressUpdatePayload::SetStartBlock(start_block) => {
                            state[id].start_block = start_block;
                        }
                    }
                }
            }
        });

        if let Some(prometheus) = &mut self.prometheus {
            prometheus.start();
        }
    }

    pub fn setup_prometheus(&mut self, address: String) {
        let receiver = self.sender.subscribe();
        self.prometheus = Some(Prometheus::new(address, receiver));
    }

    pub async fn create_progress(&self, label: String) -> ProgressChannel {
        let state = ProgressState { label, start_block: 0, current_block: 0, end_block: 0 };
        let id = self.state.lock().await.len();
        self.state.lock().await.push(state);
        ProgressChannel::new(id, self.sender.clone())
    }
}

#[derive(Clone)]
pub struct ProgressChannel {
    id: usize,
    channel: broadcast::Sender<(usize, ProgressUpdatePayload)>,
}

impl ProgressChannel {
    pub fn new(id: usize, channel: broadcast::Sender<(usize, ProgressUpdatePayload)>) -> Self {
        Self { id, channel }
    }

    pub async fn send(&self, payload: ProgressUpdatePayload) {
        self.channel.send((self.id, payload)).expect("Failed to send progress update");
    }
}
