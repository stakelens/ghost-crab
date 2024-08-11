use super::progress::ProgressState;
use axum::{extract::State, response::Json, routing::get, Router};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn init_server(state: Arc<Mutex<Vec<ProgressState>>>) {
    let app = Router::new().route("/", get(handler)).with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn handler(State(state): State<Arc<Mutex<Vec<ProgressState>>>>) -> Json<Vec<ProgressState>> {
    let state = state.lock().await;
    Json(state.clone())
}
