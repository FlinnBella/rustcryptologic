use cryptonode::{
    Result,
    bluetooth::BluetoothManager,
    wallet::WalletManager,
    bandwidth::BandwidthManager,
    config::ConfigManager,
    types::CurrencyType,
};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .build();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting CryptoNode...");

    // Initialize configuration
    let config_manager = ConfigManager::new().await?;
    let config = config_manager.get_config().await?;
    info!("Configuration loaded successfully");

    // Initialize wallet manager
    let wallet_manager = Arc::new(WalletManager::new());
    info!("Wallet manager initialized");

    // Initialize bandwidth manager
    let bandwidth_manager = BandwidthManager::new(wallet_manager.clone());
    info!("Bandwidth manager initialized");

    // Initialize Bluetooth
    let (bluetooth_manager, mut bluetooth_events) = BluetoothManager::new().await?;
    info!("Bluetooth manager initialized");

    // Start Bluetooth scanning
    bluetooth_manager.start_scan().await?;
    info!("Bluetooth scanning started");

    // Create default wallet if none exists
    let wallets = wallet_manager.list_wallets().await?;
    if wallets.is_empty() {
        info!("Creating default wallet...");
        let wallet = wallet_manager.create_wallet(CurrencyType::Bitcoin).await?;
        info!("Created default wallet with ID: {}", wallet.id);

        // Start bandwidth monitoring for the default wallet
        bandwidth_manager.start_monitoring(wallet.id).await?;
        info!("Bandwidth monitoring started for wallet: {}", wallet.id);
    }

    // Main event loop
    info!("Entering main event loop...");
    loop {
        tokio::select! {
            // Handle Bluetooth events
            Some(event) = bluetooth_events.recv() => {
                match event {
                    cryptonode::bluetooth::BluetoothEvent::DeviceDiscovered(name) => {
                        info!("Discovered Bluetooth device: {}", name);
                    }
                    cryptonode::bluetooth::BluetoothEvent::DeviceConnected(name) => {
                        info!("Connected to Bluetooth device: {}", name);
                    }
                    cryptonode::bluetooth::BluetoothEvent::DeviceDisconnected(name) => {
                        info!("Disconnected from Bluetooth device: {}", name);
                    }
                    cryptonode::bluetooth::BluetoothEvent::DataReceived(data) => {
                        info!("Received {} bytes of data", data.len());
                        // Handle received data
                        // TODO: Implement command processing
                    }
                    cryptonode::bluetooth::BluetoothEvent::Error(err) => {
                        error!("Bluetooth error: {}", err);
                    }
                }
            }

            // Handle system signals
            _ = signal::ctrl_c() => {
                info!("Received shutdown signal");
                break;
            }
        }
    }

    // Cleanup
    info!("Shutting down...");
    bluetooth_manager.disconnect().await?;
    info!("Bluetooth disconnected");

    Ok(())
} 