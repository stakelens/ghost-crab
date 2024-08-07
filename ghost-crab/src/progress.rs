use tokio::sync::mpsc::{self, Receiver, Sender};

pub struct ProgressState {
    pub label: String,
    pub start_block: u64,
    pub current_block: u64,
    pub end_block: u64,
    pub channel: Receiver<ProgressUpdate>,
}

pub enum ProgressUpdate {
    IncrementCurrentBlock,
    UpdateEndBlock(u64),
}

pub struct ProgressManager {
    progress_states: Vec<ProgressState>,
}

impl ProgressManager {
    pub fn new() -> ProgressManager {
        ProgressManager { progress_states: Vec::new() }
    }

    pub fn create_progress(&mut self, label: String, start_block: u64) -> Sender<ProgressUpdate> {
        let (tx, rx) = mpsc::channel::<ProgressUpdate>(10_000);

        let progress_state =
            ProgressState { label, start_block, current_block: 0, end_block: 0, channel: rx };

        self.progress_states.push(progress_state);

        return tx;
    }

    pub fn start(&mut self) {
        let progress_states = std::mem::replace(&mut self.progress_states, Vec::new());

        for mut progress_state in progress_states {
            tokio::spawn(async move {
                while let Some(message) = progress_state.channel.recv().await {
                    match message {
                        ProgressUpdate::IncrementCurrentBlock => {
                            progress_state.current_block += 1;
                        }
                        ProgressUpdate::UpdateEndBlock(end_block) => {
                            progress_state.end_block = end_block;
                        }
                    }
                }
            });
        }
    }
}
