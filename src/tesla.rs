use crate::{UsbDevice, TeslaConfig, PartitionConfig};
use anyhow::Result;
use std::path::Path;
use tokio::fs;

pub async fn format_for_tesla(device: &UsbDevice, config: &TeslaConfig) -> Result<()> {
    let partitions = create_tesla_partitions(config);
    
    crate::partitions::create_partitions(device, &partitions).await?;
    
    setup_tesla_folders(device, config).await?;
    
    Ok(())
}

fn create_tesla_partitions(config: &TeslaConfig) -> Vec<PartitionConfig> {
    let mut partitions = Vec::new();
    
    if config.dashcam_size_gb > 0 {
        partitions.push(PartitionConfig {
            name: "TeslaCam".to_string(),
            size_gb: config.dashcam_size_gb,
            filesystem: "exfat".to_string(),
            purpose: "Dashcam and Sentry Mode recordings".to_string(),
        });
    }
    
    if config.music_size_gb > 0 {
        partitions.push(PartitionConfig {
            name: "TeslaMusic".to_string(),
            size_gb: config.music_size_gb,
            filesystem: "exfat".to_string(),
            purpose: "Music files".to_string(),
        });
    }
    
    if config.lightshow_size_gb > 0 {
        partitions.push(PartitionConfig {
            name: "TeslaLightshow".to_string(),
            size_gb: config.lightshow_size_gb,
            filesystem: "exfat".to_string(),
            purpose: "Lightshow files".to_string(),
        });
    }
    
    partitions
}

async fn setup_tesla_folders(device: &UsbDevice, _config: &TeslaConfig) -> Result<()> {
    let mount_points = get_device_mount_points(device).await?;
    
    for mount_point in mount_points {
        let mount_path = Path::new(&mount_point);
        
        if mount_path.file_name().unwrap_or_default().to_string_lossy().contains("TeslaCam") {
            setup_dashcam_folders(&mount_point).await?;
        } else if mount_path.file_name().unwrap_or_default().to_string_lossy().contains("TeslaMusic") {
            setup_music_folders(&mount_point).await?;
        } else if mount_path.file_name().unwrap_or_default().to_string_lossy().contains("TeslaLightshow") {
            setup_lightshow_folders(&mount_point).await?;
        }
    }
    
    Ok(())
}

async fn setup_dashcam_folders(mount_point: &str) -> Result<()> {
    let base_path = Path::new(mount_point);
    
    let teslacam_path = base_path.join("TeslaCam");
    fs::create_dir_all(&teslacam_path).await?;
    
    let saved_clips_path = teslacam_path.join("SavedClips");
    fs::create_dir_all(&saved_clips_path).await?;
    
    let sentry_clips_path = teslacam_path.join("SentryClips");
    fs::create_dir_all(&sentry_clips_path).await?;
    
    let recent_clips_path = teslacam_path.join("RecentClips");
    fs::create_dir_all(&recent_clips_path).await?;
    
    Ok(())
}

async fn setup_music_folders(mount_point: &str) -> Result<()> {
    let base_path = Path::new(mount_point);
    
    let music_path = base_path.join("Music");
    fs::create_dir_all(&music_path).await?;
    
    Ok(())
}

async fn setup_lightshow_folders(mount_point: &str) -> Result<()> {
    let base_path = Path::new(mount_point);
    
    let lightshow_path = base_path.join("LightShow");
    fs::create_dir_all(&lightshow_path).await?;
    
    Ok(())
}

async fn get_device_mount_points(device: &UsbDevice) -> Result<Vec<String>> {
    let mut mount_points = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        mount_points.push(format!("{}\\", device.path));
    }
    
    #[cfg(target_os = "macos")]
    {
        let output = tokio::process::Command::new("mount")
            .output()
            .await?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains(&device.path) {
                if let Some(mount_point) = line.split(" on ").nth(1) {
                    if let Some(mount_path) = mount_point.split(" (").next() {
                        mount_points.push(mount_path.to_string());
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        let output = tokio::process::Command::new("mount")
            .output()
            .await?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains(&device.path) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    mount_points.push(parts[2].to_string());
                }
            }
        }
    }
    
    Ok(mount_points)
}

pub fn get_tesla_requirements() -> TeslaRequirements {
    TeslaRequirements {
        min_total_size_gb: 32,
        min_dashcam_size_gb: 32,
        recommended_write_speed_mbps: 4,
        supported_filesystems: vec![
            "exfat".to_string(),
            "fat32".to_string(),
            "ext3".to_string(),
            "ext4".to_string(),
        ],
        required_folders: vec![
            "TeslaCam".to_string(),
            "TeslaCam/SavedClips".to_string(),
            "TeslaCam/SentryClips".to_string(),
            "TeslaCam/RecentClips".to_string(),
        ],
    }
}

#[derive(Debug, Clone)]
pub struct TeslaRequirements {
    pub min_total_size_gb: u32,
    pub min_dashcam_size_gb: u32,
    pub recommended_write_speed_mbps: u32,
    pub supported_filesystems: Vec<String>,
    pub required_folders: Vec<String>,
}

pub fn validate_tesla_config(device: &UsbDevice, config: &TeslaConfig) -> Result<()> {
    let requirements = get_tesla_requirements();
    let device_size_gb = device.size / (1024 * 1024 * 1024);
    
    if device_size_gb < requirements.min_total_size_gb as u64 {
        return Err(anyhow::anyhow!(
            "Device size ({} GB) is below Tesla minimum requirement ({} GB)",
            device_size_gb,
            requirements.min_total_size_gb
        ));
    }
    
    if config.dashcam_size_gb < requirements.min_dashcam_size_gb {
        return Err(anyhow::anyhow!(
            "Dashcam partition size ({} GB) is below Tesla minimum requirement ({} GB)",
            config.dashcam_size_gb,
            requirements.min_dashcam_size_gb
        ));
    }
    
    let total_config_size = config.dashcam_size_gb + config.sentry_size_gb + 
                           config.music_size_gb + config.lightshow_size_gb;
    
    if total_config_size as u64 > device_size_gb {
        return Err(anyhow::anyhow!(
            "Total configured size ({} GB) exceeds device capacity ({} GB)",
            total_config_size,
            device_size_gb
        ));
    }
    
    Ok(())
}

pub fn get_recommended_tesla_config(device_size_gb: u32) -> TeslaConfig {
    if device_size_gb < 64 {
        TeslaConfig {
            dashcam_size_gb: 32,
            sentry_size_gb: 0,
            music_size_gb: 0,
            lightshow_size_gb: 0,
        }
    } else if device_size_gb < 128 {
        TeslaConfig {
            dashcam_size_gb: 32,
            sentry_size_gb: 0,
            music_size_gb: 16,
            lightshow_size_gb: 8,
        }
    } else {
        TeslaConfig {
            dashcam_size_gb: 64,
            sentry_size_gb: 0,
            music_size_gb: 32,
            lightshow_size_gb: 16,
        }
    }
}