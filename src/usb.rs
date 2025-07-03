use crate::UsbDevice;
use anyhow::Result;
use tokio::process::Command as TokioCommand;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub async fn list_usb_devices() -> Result<Vec<UsbDevice>> {
    let mut devices = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        devices = list_windows_devices().await?;
    }
    
    #[cfg(target_os = "macos")]
    {
        devices = list_macos_devices().await?;
    }
    
    #[cfg(target_os = "linux")]
    {
        devices = list_linux_devices().await?;
    }
    
    Ok(devices.into_iter().filter(|d| d.is_removable).collect())
}

pub async fn get_device_info(device_path: &str) -> Result<UsbDevice> {
    #[cfg(target_os = "windows")]
    {
        get_windows_device_info(device_path).await
    }
    
    #[cfg(target_os = "macos")]
    {
        get_macos_device_info(device_path).await
    }
    
    #[cfg(target_os = "linux")]
    {
        get_linux_device_info(device_path).await
    }
}

#[cfg(target_os = "windows")]
async fn list_windows_devices() -> Result<Vec<UsbDevice>> {
    let output = TokioCommand::new("wmic")
        .args(&[
            "logicaldisk",
            "where",
            "drivetype=2",
            "get",
            "size,freespace,caption",
            "/format:csv"
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .await?;
    
    let _output_str = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    
    for line in output_str.lines().skip(1) {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 4 && !fields[1].is_empty() {
            let drive_letter = fields[1].trim();
            let size_str = fields[2].trim();
            
            if let Ok(size) = size_str.parse::<u64>() {
                devices.push(UsbDevice {
                    name: format!("Removable Disk ({})", drive_letter),
                    path: drive_letter.to_string(),
                    size,
                    is_removable: true,
                });
            }
        }
    }
    
    Ok(devices)
}

#[cfg(target_os = "windows")]
async fn get_windows_device_info(device_path: &str) -> Result<UsbDevice> {
    let output = TokioCommand::new("wmic")
        .args(&[
            "logicaldisk",
            "where",
            &format!("caption='{}'", device_path),
            "get",
            "size,freespace,caption,volumename",
            "/format:csv"
        ])
        .creation_flags(0x08000000)
        .output()
        .await?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    
    if lines.len() >= 2 {
        let fields: Vec<&str> = lines[1].split(',').collect();
        if fields.len() >= 4 {
            let size = fields[2].trim().parse::<u64>().unwrap_or(0);
            let volume_name = fields[3].trim();
            let name = if volume_name.is_empty() {
                format!("Removable Disk ({})", device_path)
            } else {
                format!("{} ({})", volume_name, device_path)
            };
            
            return Ok(UsbDevice {
                name,
                path: device_path.to_string(),
                size,
                is_removable: true,
            });
        }
    }
    
    Err(anyhow::anyhow!("Device not found"))
}

#[cfg(target_os = "macos")]
async fn list_macos_devices() -> Result<Vec<UsbDevice>> {
    let mut devices = Vec::new();
    
    let external_output = TokioCommand::new("diskutil")
        .args(&["list", "external"])
        .output()
        .await?;
    
    let external_str = String::from_utf8_lossy(&external_output.stdout);
    
    for line in external_str.lines() {
        if line.contains("/dev/disk") && !line.contains("(internal") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(disk_path) = parts.first() {
                if let Ok(info) = get_macos_device_info(disk_path).await {
                    devices.push(info);
                }
            }
        }
    }
    
    Ok(devices)
}

#[cfg(target_os = "macos")]
async fn get_macos_device_info(device_path: &str) -> Result<UsbDevice> {
    let output = TokioCommand::new("diskutil")
        .args(&["info", device_path])
        .output()
        .await?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut name = String::new();
    let mut size = 0u64;
    let mut is_removable = false;
    
    for line in output_str.lines() {
        if line.contains("Device / Media Name:") {
            name = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Disk Size:") {
            if let Some(size_part) = line.split('(').nth(1) {
                if let Some(bytes_str) = size_part.split_whitespace().next() {
                    size = bytes_str.parse::<u64>().unwrap_or(0);
                }
            }
        } else if line.contains("Removable Media:") {
            is_removable = line.contains("Yes");
        }
    }
    
    if name.is_empty() {
        name = format!("USB Device {}", device_path);
    }
    
    Ok(UsbDevice {
        name,
        path: device_path.to_string(),
        size,
        is_removable,
    })
}

#[cfg(target_os = "linux")]
async fn list_linux_devices() -> Result<Vec<UsbDevice>> {
    let output = TokioCommand::new("lsblk")
        .args(&["-J", "-o", "NAME,SIZE,HOTPLUG,MOUNTPOINT,TYPE"])
        .output()
        .await?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    
    for line in output_str.lines() {
        if line.contains("\"hotplug\":true") && line.contains("\"type\":\"disk\"") {
            let usb_output = TokioCommand::new("lsblk")
                .args(&["-n", "-o", "NAME,SIZE,HOTPLUG"])
                .output()
                .await?;
            
            let usb_str = String::from_utf8_lossy(&usb_output.stdout);
            for usb_line in usb_str.lines() {
                let parts: Vec<&str> = usb_line.split_whitespace().collect();
                if parts.len() >= 3 && parts[2] == "1" {
                    let device_name = parts[0];
                    let size_str = parts[1];
                    let device_path = format!("/dev/{}", device_name);
                    
                    if let Ok(info) = get_linux_device_info(&device_path).await {
                        devices.push(info);
                    }
                }
            }
        }
    }
    
    Ok(devices)
}

#[cfg(target_os = "linux")]
async fn get_linux_device_info(device_path: &str) -> Result<UsbDevice> {
    let output = TokioCommand::new("lsblk")
        .args(&["-b", "-n", "-o", "SIZE,MODEL", device_path])
        .output()
        .await?;
    
    let _output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.trim().split_whitespace().collect();
    
    let size = if !parts.is_empty() {
        parts[0].parse::<u64>().unwrap_or(0)
    } else {
        0
    };
    
    let model = if parts.len() > 1 {
        parts[1..].join(" ")
    } else {
        "USB Device".to_string()
    };
    
    Ok(UsbDevice {
        name: model,
        path: device_path.to_string(),
        size,
        is_removable: true,
    })
}