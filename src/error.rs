use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum CryptoNodeError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Bluetooth error: {0}")]
    Bluetooth(String),

    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Bandwidth error: {0}")]
    Bandwidth(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Crypto operation error: {0}")]
    CryptoOperation(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Device error: {0}")]
    Device(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Operation timeout")]
    Timeout,

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource busy: {0}")]
    ResourceBusy(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
} 