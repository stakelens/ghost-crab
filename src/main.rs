mod cache;
mod config;
mod db;
mod handlers;
mod indexer;
mod manager;
mod models;
mod schema;
use manager::Indexer;

#[tokio::main]
async fn main() {
    let mut indexer = Indexer::new();
    indexer.load(handlers::rocketpool::MinipoolCreatedHandler::new());
    indexer.start().await;
}
