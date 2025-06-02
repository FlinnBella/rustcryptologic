pub mod bluetooth;
pub mod crypto;
pub mod wallet;
pub mod bandwidth;
pub mod storage;
pub mod config;
pub mod error;
pub mod types;

use error::CryptoNodeError;
pub type Result<T> = std::result::Result<T, CryptoNodeError>;

/// Re-export commonly used types
pub use types::{
    Transaction,
    Wallet,
    BandwidthMetrics,
    DeviceConfig,
    ConnectionStatus,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Feature flags
#[cfg(feature = "bluetooth")]
pub const BLUETOOTH_ENABLED: bool = true;
#[cfg(not(feature = "bluetooth"))]
pub const BLUETOOTH_ENABLED: bool = false;

#[cfg(feature = "crypto")]
pub const CRYPTO_ENABLED: bool = true;
#[cfg(not(feature = "crypto"))]
pub const CRYPTO_ENABLED: bool = false;

#[cfg(feature = "bandwidth")]
pub const BANDWIDTH_ENABLED: bool = true;
#[cfg(not(feature = "bandwidth"))]
pub const BANDWIDTH_ENABLED: bool = false; 