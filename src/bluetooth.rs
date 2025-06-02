use crate::{Result, error::CryptoNodeError};
use btleplug::api::{
    Central, CentralEvent, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use tokio::sync::mpsc;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service UUID for our custom BLE service
pub const SERVICE_UUID: Uuid = Uuid::from_u128(0x12345678_1234_1234_1234_123456789ABC);

/// Characteristic UUIDs for different operations
pub const CHARACTERISTIC_UUIDS: &[Uuid] = &[
    Uuid::from_u128(0x12345678_1234_1234_1234_123456789ABC), // Command
    Uuid::from_u128(0x12345678_1234_1234_1234_123456789ABD), // Response
    Uuid::from_u128(0x12345678_1234_1234_1234_123456789ABE), // Notification
];

/// Represents a Bluetooth connection manager
pub struct BluetoothManager {
    adapter: Adapter,
    characteristics: Arc<RwLock<Vec<Characteristic>>>,
    connected_device: Arc<RwLock<Option<Peripheral>>>,
    event_sender: mpsc::Sender<BluetoothEvent>,
}

/// Events that can occur during Bluetooth operation
#[derive(Debug, Clone)]
pub enum BluetoothEvent {
    DeviceDiscovered(String),
    DeviceConnected(String),
    DeviceDisconnected(String),
    DataReceived(Vec<u8>),
    Error(String),
}

impl BluetoothManager {
    /// Create a new Bluetooth manager
    pub async fn new() -> Result<(Self, mpsc::Receiver<BluetoothEvent>)> {
        let manager = Manager::new().await.map_err(|e| CryptoNodeError::Bluetooth(e.to_string()))?;
        let adapters = manager.adapters().await.map_err(|e| CryptoNodeError::Bluetooth(e.to_string()))?;
        let adapter = adapters.into_iter().next()
            .ok_or_else(|| CryptoNodeError::Bluetooth("No Bluetooth adapter found".to_string()))?;

        let (tx, rx) = mpsc::channel(100);

        Ok((Self {
            adapter,
            characteristics: Arc::new(RwLock::new(Vec::new())),
            connected_device: Arc::new(RwLock::new(None)),
            event_sender: tx,
        }, rx))
    }

    /// Start scanning for devices
    pub async fn start_scan(&self) -> Result<()> {
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|e| CryptoNodeError::Bluetooth(e.to_string()))?;

        let event_sender = self.event_sender.clone();
        let adapter = self.adapter.clone();

        tokio::spawn(async move {
            let mut events = adapter.events().await.unwrap();
            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        if let Ok(device) = adapter.peripheral(&id).await {
                            if let Ok(props) = device.properties().await {
                                if let Some(name) = props.local_name {
                                    let _ = event_sender.send(BluetoothEvent::DeviceDiscovered(name)).await;
                                }
                            }
                        }
                    }
                    CentralEvent::DeviceConnected(id) => {
                        if let Ok(device) = adapter.peripheral(&id).await {
                            if let Ok(props) = device.properties().await {
                                if let Some(name) = props.local_name {
                                    let _ = event_sender.send(BluetoothEvent::DeviceConnected(name)).await;
                                }
                            }
                        }
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        if let Ok(device) = adapter.peripheral(&id).await {
                            if let Ok(props) = device.properties().await {
                                if let Some(name) = props.local_name {
                                    let _ = event_sender.send(BluetoothEvent::DeviceDisconnected(name)).await;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Connect to a specific device
    pub async fn connect_to_device(&self, device: Peripheral) -> Result<()> {
        device.connect().await
            .map_err(|e| CryptoNodeError::Bluetooth(e.to_string()))?;

        device.discover_services().await
            .map_err(|e| CryptoNodeError::Bluetooth(e.to_string()))?;

        let chars = device.characteristics();
        let mut characteristics = self.characteristics.write().await;
        *characteristics = chars.into_iter()
            .filter(|c| CHARACTERISTIC_UUIDS.contains(&c.uuid))
            .collect();

        let mut connected = self.connected_device.write().await;
        *connected = Some(device);

        Ok(())
    }

    /// Send data to the connected device
    pub async fn send_data(&self, data: &[u8]) -> Result<()> {
        let device = self.connected_device.read().await;
        let device = device.as_ref()
            .ok_or_else(|| CryptoNodeError::Bluetooth("No device connected".to_string()))?;

        let characteristics = self.characteristics.read().await;
        let command_char = characteristics.iter()
            .find(|c| c.uuid == CHARACTERISTIC_UUIDS[0])
            .ok_or_else(|| CryptoNodeError::Bluetooth("Command characteristic not found".to_string()))?;

        device.write(command_char, data, WriteType::WithResponse).await
            .map_err(|e| CryptoNodeError::Bluetooth(e.to_string()))?;

        Ok(())
    }

    /// Subscribe to notifications from the device
    pub async fn subscribe_notifications(&self) -> Result<()> {
        let device = self.connected_device.read().await;
        let device = device.as_ref()
            .ok_or_else(|| CryptoNodeError::Bluetooth("No device connected".to_string()))?;

        let characteristics = self.characteristics.read().await;
        let notify_char = characteristics.iter()
            .find(|c| c.uuid == CHARACTERISTIC_UUIDS[2])
            .ok_or_else(|| CryptoNodeError::Bluetooth("Notification characteristic not found".to_string()))?;

        device.subscribe(notify_char).await
            .map_err(|e| CryptoNodeError::Bluetooth(e.to_string()))?;

        let event_sender = self.event_sender.clone();
        let device_clone = device.clone();
        
        tokio::spawn(async move {
            let mut notification_stream = device_clone.notifications().await.unwrap();
            while let Some(data) = notification_stream.next().await {
                let _ = event_sender.send(BluetoothEvent::DataReceived(data.value)).await;
            }
        });

        Ok(())
    }

    /// Disconnect from the current device
    pub async fn disconnect(&self) -> Result<()> {
        let mut device = self.connected_device.write().await;
        if let Some(d) = device.take() {
            d.disconnect().await
                .map_err(|e| CryptoNodeError::Bluetooth(e.to_string()))?;
        }
        Ok(())
    }
} 