use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Represents a cryptocurrency wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub address: String,
    pub public_key: Vec<u8>,
    #[serde(skip_serializing)]
    pub private_key: Vec<u8>,
    pub currency_type: CurrencyType,
    pub balance: f64,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Supported cryptocurrency types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrencyType {
    Bitcoin,
    Ethereum,
    // Add more currencies as needed
}

/// Represents a cryptocurrency transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub from_wallet: String,
    pub to_wallet: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    pub fee: Option<f64>,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Bandwidth sharing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthMetrics {
    pub total_shared: u64,
    pub current_rate: f64,
    pub uptime: chrono::Duration,
    pub rewards: HashMap<CurrencyType, f64>,
    pub last_updated: DateTime<Utc>,
}

/// Device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub device_id: Uuid,
    pub bluetooth_name: String,
    pub max_bandwidth: u64,
    pub min_reward_rate: f64,
    pub supported_currencies: Vec<CurrencyType>,
    pub auto_update: bool,
}

/// Bluetooth connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Pairing,
    Error,
}

/// Bandwidth sharing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthSettings {
    pub enabled: bool,
    pub max_share_percentage: f64,
    pub min_bandwidth_reserve: u64,
    pub preferred_currencies: Vec<CurrencyType>,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub require_pin: bool,
    pub auto_lock_duration: chrono::Duration,
    pub enable_biometrics: bool,
    pub backup_enabled: bool,
}

/// Device status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub connection: ConnectionStatus,
    pub battery_level: f32,
    pub storage_used: f64,
    pub current_bandwidth: f64,
    pub temperature: f32,
    pub last_sync: DateTime<Utc>,
}

/// API Response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
} 