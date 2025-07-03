#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod usb;
mod partitions;
mod tesla;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionConfig {
    pub name: String,
    pub size_gb: u32,
    pub filesystem: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeslaConfig {
    pub dashcam_size_gb: u32,
    pub sentry_size_gb: u32,
    pub music_size_gb: u32,
    pub lightshow_size_gb: u32,
}

type DeviceState = Mutex<HashMap<String, UsbDevice>>;

#[tauri::command]
async fn get_usb_devices(state: State<'_, DeviceState>) -> Result<Vec<UsbDevice>, String> {
    let devices = usb::list_usb_devices().await.map_err(|e| e.to_string())?;
    
    let mut device_map = state.lock().await;
    device_map.clear();
    for device in &devices {
        device_map.insert(device.path.clone(), device.clone());
    }
    
    Ok(devices)
}

#[tauri::command]
async fn format_tesla_usb(
    device_path: String,
    config: TeslaConfig,
    state: State<'_, DeviceState>,
) -> Result<String, String> {
    let device_map = state.lock().await;
    let device = device_map.get(&device_path)
        .ok_or("Device not found".to_string())?;
    
    tesla::format_for_tesla(device, &config)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok("USB formatted successfully for Tesla".to_string())
}

#[tauri::command]
async fn create_custom_partitions(
    device_path: String,
    partitions: Vec<PartitionConfig>,
    state: State<'_, DeviceState>,
) -> Result<String, String> {
    let device_map = state.lock().await;
    let device = device_map.get(&device_path)
        .ok_or("Device not found".to_string())?;
    
    partitions::create_partitions(device, &partitions)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok("Partitions created successfully".to_string())
}

#[tauri::command]
async fn get_device_info(device_path: String) -> Result<UsbDevice, String> {
    usb::get_device_info(&device_path)
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(DeviceState::default())
        .invoke_handler(tauri::generate_handler![
            get_usb_devices,
            format_tesla_usb,
            create_custom_partitions,
            get_device_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
