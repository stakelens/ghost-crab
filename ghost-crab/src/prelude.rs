pub use crate::event_handler::{EventContext, EventHandler};
pub use alloy;
pub use alloy::{
    sol,
    sol_types::{SolEvent, SolEventInterface},
};
pub use async_trait::async_trait;
pub use config::ExecutionMode;
pub use ghost_crab_macros::block_handler;
pub use ghost_crab_macros::event_handler;
pub use ghost_crab_macros::template;
pub use std::sync::Arc;
pub use tokio;

pub use crate::block_handler::{BlockContext, BlockHandler};
pub use crate::config;
pub use crate::indexer;
pub use crate::indexer::templates::Template;
pub use alloy::primitives::address;
pub use alloy::primitives::Address;
pub use alloy::providers::Provider;
