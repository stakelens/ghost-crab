use axum::{
    body::Body,
    extract::State,
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use prometheus_client::{
    encoding::text::encode,
    metrics::{counter::Counter, family::Family, gauge::Gauge},
    registry::Registry,
};
use prometheus_client_derive_encode::EncodeLabelSet;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProgressLabels {
    pub label: String,
}

#[derive(Debug)]
pub struct Metrics {
    start_block: Family<ProgressLabels, Gauge>,
    processed_blocks: Family<ProgressLabels, Counter>,
    end_block: Family<ProgressLabels, Gauge>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            start_block: Family::default(),
            processed_blocks: Family::default(),
            end_block: Family::default(),
        }
    }

    fn initialize_processed_blocks(&self, label: &str) {
        let _ = self.processed_blocks.get_or_create(&ProgressLabels { label: label.to_string() });
    }

    fn update_start_block(&self, label: &str, value: u64) {
        self.start_block
            .get_or_create(&ProgressLabels { label: label.to_string() })
            .set(value as i64);
    }

    fn increment_processed_blocks(&self, label: &str, count: u64) {
        self.processed_blocks
            .get_or_create(&ProgressLabels { label: label.to_string() })
            .inc_by(count);
    }

    fn update_end_block(&self, label: &str, value: u64) {
        self.end_block
            .get_or_create(&ProgressLabels { label: label.to_string() })
            .set(value as i64);
    }
}

#[derive(Debug)]
pub struct AppState {
    pub registry: Registry,
    pub metrics: Arc<Metrics>,
}

#[derive(Clone)]
pub enum ProgressUpdatePayload {
    IncrementProcessedBlocks(u64),
    UpdateEndBlock(u64),
}

pub struct ProgressManager {
    state: Arc<Mutex<AppState>>,
    indexer_sender: mpsc::Sender<(String, ProgressUpdatePayload)>,
}

impl ProgressManager {
    pub fn new() -> Self {
        let metrics = Arc::new(Metrics::new());
        let mut registry = Registry::default();

        registry.register(
            "start_block",
            "Starting block for indexing",
            metrics.start_block.clone(),
        );
        registry.register(
            "processed_blocks",
            "Number of blocks processed",
            metrics.processed_blocks.clone(),
        );
        registry.register("end_block", "End block for indexing", metrics.end_block.clone());

        let state = Arc::new(Mutex::new(AppState { registry, metrics }));

        let (indexer_sender, mut indexer_receiver) =
            mpsc::channel::<(String, ProgressUpdatePayload)>(10_000);

        let update_state = Arc::clone(&state);
        tokio::spawn(async move {
            while let Some((label, update)) = indexer_receiver.recv().await {
                let state = update_state.lock().await;
                match update {
                    ProgressUpdatePayload::IncrementProcessedBlocks(count) => {
                        state.metrics.increment_processed_blocks(&label, count);
                    }
                    ProgressUpdatePayload::UpdateEndBlock(block) => {
                        state.metrics.update_end_block(&label, block);
                    }
                }
            }
        });

        ProgressManager { state, indexer_sender }
    }

    pub async fn create_progress(&self, label: String, start_block: u64) -> ProgressChannel {
        let state = self.state.lock().await;
        state.metrics.initialize_processed_blocks(&label);
        state.metrics.update_start_block(&label, start_block);
        state.metrics.update_end_block(&label, start_block);

        ProgressChannel::new(label, self.indexer_sender.clone())
    }

    pub fn start(&self) {
        let state = self.state.clone();
        tokio::spawn(async move {
            let app = Router::new().route("/metrics", get(metrics_handler)).with_state(state);

            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            println!("Metrics server listening on {}", listener.local_addr().unwrap());

            axum::serve(listener, app).await.unwrap();
        });
    }
}

#[derive(Clone)]
pub struct ProgressChannel {
    label: String,
    channel: mpsc::Sender<(String, ProgressUpdatePayload)>,
}

impl ProgressChannel {
    pub fn new(label: String, channel: mpsc::Sender<(String, ProgressUpdatePayload)>) -> Self {
        Self { label, channel }
    }

    pub async fn send(&self, payload: ProgressUpdatePayload) {
        self.channel
            .send((self.label.clone(), payload))
            .await
            .expect("Failed to send progress update");
    }
}

async fn metrics_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let mut buffer = String::new();
    encode(&mut buffer, &state.registry).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(buffer))
        .unwrap()
}
