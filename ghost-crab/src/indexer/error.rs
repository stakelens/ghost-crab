use alloy::hex::FromHexError;
use core::fmt;

#[derive(Debug)]
pub enum AddHandlerError {
    NotFound(String),
    NetworkNotFound(String),
    InvalidAddress { address: String, error: FromHexError },
}

impl fmt::Display for AddHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddHandlerError::NotFound(handler) => {
                write!(f, "Handler not found: {}", handler)
            }
            AddHandlerError::NetworkNotFound(network) => {
                write!(f, "Network not found: {}", network)
            }
            AddHandlerError::InvalidAddress { address, error } => {
                write!(f, "Invalid address: {}.\nError: {}", address, error)
            }
        }
    }
}

impl std::error::Error for AddHandlerError {}
