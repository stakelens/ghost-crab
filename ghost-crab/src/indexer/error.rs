use core::fmt;

#[derive(Debug)]
pub enum AddHandlerError {
    NotFound(String),
    NetworkNotFound(String),
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
        }
    }
}

impl std::error::Error for AddHandlerError {}
