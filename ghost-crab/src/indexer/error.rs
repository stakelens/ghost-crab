use alloy::hex::FromHexError;
use core::fmt;

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    DB(rocksdb::Error),
    NetworkNotFound(String),
    InvalidAddress(FromHexError),
    CacheFileNotFound(std::io::Error),
    InvalidRpcUrl(Box<dyn std::error::Error>),
}

pub type Result<T> = core::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotFound(handler) => {
                writeln!(f, "Handler not found: {}", handler)
            }
            Error::DB(error) => {
                writeln!(f, "Error while loading cache: {}", error)
            }
            Error::NetworkNotFound(network) => {
                writeln!(f, "Network not found: {}", network)
            }
            Error::InvalidAddress(error) => {
                writeln!(f, "Invalid address: {}", error)
            }
            Error::CacheFileNotFound(error) => {
                writeln!(f, "Cache file not found: {}", error)
            }
            Error::InvalidRpcUrl(error) => {
                writeln!(f, "Invalid RPC url: {}", error)
            }
        }
    }
}

impl std::error::Error for Error {}
