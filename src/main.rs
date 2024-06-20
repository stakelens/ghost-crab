mod db;
mod handlers;
mod indexer;
mod models;
mod cache;
mod schema;
mod manager;
use manager::Indexer;

#[tokio::main]
async fn main() {
    let mut indexer = Indexer::load();
    indexer.add(handlers::rocketpool::MinipoolCreatedHandler::new());
    indexer.start().await;
}
