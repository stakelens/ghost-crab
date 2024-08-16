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
use tokio::sync::{broadcast, Mutex};

use super::progress::ProgressUpdatePayload;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProgressLabels {
    pub label: usize,
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
            end_block: Family::default(),
            start_block: Family::default(),
            processed_blocks: Family::default(),
        }
    }

    fn set_start_block(&self, label: usize, value: u64) {
        self.start_block.get_or_create(&ProgressLabels { label }).set(value as i64);
    }

    fn increment_processed_blocks(&self, label: usize, count: u64) {
        self.processed_blocks.get_or_create(&ProgressLabels { label }).inc_by(count);
    }

    fn update_end_block(&self, label: usize, value: u64) {
        self.end_block.get_or_create(&ProgressLabels { label }).set(value as i64);
    }
}

#[derive(Debug)]
pub struct PrometheusState {
    pub registry: Registry,
    pub metrics: Arc<Metrics>,
}

pub struct Prometheus {
    address: String,
    state: Arc<Mutex<PrometheusState>>,
    receiver: Option<broadcast::Receiver<(usize, ProgressUpdatePayload)>>,
}

impl Prometheus {
    pub fn new(
        address: String,
        receiver: broadcast::Receiver<(usize, ProgressUpdatePayload)>,
    ) -> Prometheus {
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

        let state = Arc::new(Mutex::new(PrometheusState { registry, metrics }));
        Prometheus { address, state, receiver: Some(receiver) }
    }

    pub fn start(&mut self) {
        let state = Arc::clone(&self.state);
        let address = self.address.clone();
        let mut receiver = self.receiver.take().expect("ProccessManager already started");

        tokio::spawn(async move {
            loop {
                if let Ok((id, payload)) = receiver.recv().await {
                    let state = Arc::clone(&state);
                    let state = state.lock().await;

                    match payload {
                        ProgressUpdatePayload::IncrementProcessedBlocks(amount) => {
                            state.metrics.increment_processed_blocks(id, amount);
                        }
                        ProgressUpdatePayload::UpdateEndBlock(end_block) => {
                            state.metrics.update_end_block(id, end_block);
                        }
                        ProgressUpdatePayload::SetStartBlock(start_block) => {
                            state.metrics.set_start_block(id, start_block);
                        }
                    }
                }
            }
        });

        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            let app = Router::new().route("/metrics", get(metrics_handler)).with_state(state);
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            println!("Metrics server listening on {:?}", listener);
            axum::serve(listener, app).await.unwrap();
        });
    }
}

async fn metrics_handler(State(state): State<Arc<Mutex<PrometheusState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let mut buffer = String::new();
    encode(&mut buffer, &state.registry).unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(buffer))
        .unwrap()
}
