pub use crate::cache::manager::RPC_MANAGER;
pub use crate::handler::{Context, Handler, HandlerConfig};
pub use alloy;
pub use alloy::{
    sol,
    sol_types::{SolEvent, SolEventInterface},
};
pub use async_trait::async_trait;
pub use indexer_macros::block_handler;
pub use indexer_macros::handler;
pub use std::sync::Arc;
pub use tokio;

pub use crate::block_handler::{BlockContext, BlockHandler};
pub use crate::cache;
pub use crate::config;
pub use crate::indexer;
pub use crate::indexer::Template;
pub use crate::process_logs;
pub use alloy::providers::Provider;
