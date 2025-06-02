use crate::{
    Result,
    error::CryptoNodeError,
    types::DeviceConfig,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use std::sync::Arc;

const CONFIG_FILE: &str = "config.json";

/// Manages application configuration
pub struct ConfigManager {
    config: Arc<RwLock<DeviceConfig>>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub async fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CryptoNodeError::Config("Could not determine config directory".to_string()))?
            .join("cryptonode");

        fs::create_dir_all(&config_dir)
            .map_err(|e| CryptoNodeError::Config(format!("Failed to create config directory: {}", e)))?;

        let config_path = config_dir.join(CONFIG_FILE);
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            let default_config = DeviceConfig::default();
            Self::save_config(&config_path, &default_config)?;
            default_config
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        })
    }

    /// Load configuration from file
    fn load_config(path: &Path) -> Result<DeviceConfig> {
        let config_str = fs::read_to_string(path)
            .map_err(|e| CryptoNodeError::Config(format!("Failed to read config file: {}", e)))?;

        serde_json::from_str(&config_str)
            .map_err(|e| CryptoNodeError::Config(format!("Failed to parse config file: {}", e)))
    }

    /// Save configuration to file
    fn save_config(path: &Path, config: &DeviceConfig) -> Result<()> {
        let config_str = serde_json::to_string_pretty(config)
            .map_err(|e| CryptoNodeError::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, config_str)
            .map_err(|e| CryptoNodeError::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> Result<DeviceConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }

    /// Update configuration
    pub async fn update_config(&self, new_config: DeviceConfig) -> Result<()> {
        // Save to file first to ensure persistence
        Self::save_config(&self.config_path, &new_config)?;

        // Update in-memory config
        let mut config = self.config.write().await;
        *config = new_config;

        Ok(())
    }

    /// Update specific configuration field
    pub async fn update_field<T: Serialize>(&self, field: &str, value: T) -> Result<()> {
        let mut config = self.config.write().await;
        let config_value = serde_json::to_value(&config)
            .map_err(|e| CryptoNodeError::Config(format!("Failed to serialize config: {}", e)))?;

        let mut config_map = config_value.as_object()
            .ok_or_else(|| CryptoNodeError::Config("Invalid config structure".to_string()))?
            .clone();

        let value = serde_json::to_value(value)
            .map_err(|e| CryptoNodeError::Config(format!("Failed to serialize value: {}", e)))?;

        config_map.insert(field.to_string(), value);

        *config = serde_json::from_value(serde_json::Value::Object(config_map))
            .map_err(|e| CryptoNodeError::Config(format!("Failed to update config: {}", e)))?;

        Self::save_config(&self.config_path, &config)?;

        Ok(())
    }

    /// Reset configuration to defaults
    pub async fn reset_config(&self) -> Result<()> {
        let default_config = DeviceConfig::default();
        self.update_config(default_config).await
    }

    /// Get configuration file path
    pub fn get_config_path(&self) -> &Path {
        &self.config_path
    }

    /// Validate configuration
    pub async fn validate_config(&self) -> Result<()> {
        let config = self.config.read().await;
        
        // Validate device name
        if config.device_name.is_empty() {
            return Err(CryptoNodeError::Config("Device name cannot be empty".to_string()));
        }

        // Validate Bluetooth settings
        if config.bluetooth_enabled {
            if config.bluetooth_name.is_empty() {
                return Err(CryptoNodeError::Config("Bluetooth name cannot be empty when enabled".to_string()));
            }
        }

        // Validate bandwidth settings
        if config.min_bandwidth == 0 {
            return Err(CryptoNodeError::Config("Minimum bandwidth cannot be zero".to_string()));
        }

        if config.reward_rate < 0.0 {
            return Err(CryptoNodeError::Config("Reward rate cannot be negative".to_string()));
        }

        // Validate update settings
        if config.auto_update_enabled && config.update_check_interval == 0 {
            return Err(CryptoNodeError::Config("Update check interval cannot be zero when auto-update is enabled".to_string()));
        }

        Ok(())
    }

    /// Export configuration to file
    pub async fn export_config(&self, path: &Path) -> Result<()> {
        let config = self.config.read().await;
        Self::save_config(path, &config)
    }

    /// Import configuration from file
    pub async fn import_config(&self, path: &Path) -> Result<()> {
        let new_config = Self::load_config(path)?;
        self.update_config(new_config).await
    }
} 