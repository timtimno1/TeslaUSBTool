use crate::{UsbDevice, PartitionConfig};
use anyhow::Result;
use tokio::process::Command as TokioCommand;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub async fn create_partitions(device: &UsbDevice, partitions: &[PartitionConfig]) -> Result<()> {
    validate_partition_config(device, partitions)?;
    
    #[cfg(target_os = "windows")]
    {
        create_windows_partitions(device, partitions).await
    }
    
    #[cfg(target_os = "macos")]
    {
        create_macos_partitions(device, partitions).await
    }
    
    #[cfg(target_os = "linux")]
    {
        create_linux_partitions(device, partitions).await
    }
}

fn validate_partition_config(device: &UsbDevice, partitions: &[PartitionConfig]) -> Result<()> {
    let total_size: u64 = partitions.iter()
        .map(|p| p.size_gb as u64 * 1024 * 1024 * 1024)
        .sum();
    
    if total_size > device.size {
        return Err(anyhow::anyhow!(
            "Total partition size ({} GB) exceeds device capacity ({} GB)",
            total_size / (1024 * 1024 * 1024),
            device.size / (1024 * 1024 * 1024)
        ));
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
async fn create_windows_partitions(device: &UsbDevice, partitions: &[PartitionConfig]) -> Result<()> {
    let disk_part_script = create_diskpart_script(device, partitions)?;
    
    let temp_file = std::env::temp_dir().join("diskpart_script.txt");
    tokio::fs::write(&temp_file, disk_part_script).await?;
    
    let output = TokioCommand::new("diskpart")
        .args(&["/s", temp_file.to_str().unwrap()])
        .creation_flags(0x08000000)
        .output()
        .await?;
    
    tokio::fs::remove_file(&temp_file).await?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Diskpart failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
fn create_diskpart_script(device: &UsbDevice, partitions: &[PartitionConfig]) -> Result<String> {
    let mut script = String::new();
    
    script.push_str(&format!("select disk {}\n", extract_disk_number(&device.path)?));
    script.push_str("clean\n");
    script.push_str("convert gpt\n");
    
    for (i, partition) in partitions.iter().enumerate() {
        script.push_str(&format!("create partition primary size={}\n", partition.size_gb * 1024));
        script.push_str(&format!("select partition {}\n", i + 1));
        script.push_str("active\n");
        
        let format_cmd = match partition.filesystem.as_str() {
            "exfat" => "format fs=exfat quick",
            "fat32" => "format fs=fat32 quick",
            "ntfs" => "format fs=ntfs quick",
            _ => "format fs=exfat quick",
        };
        
        script.push_str(&format!("{}\n", format_cmd));
        script.push_str(&format!("assign letter={}\n", get_available_drive_letter()));
        script.push_str(&format!("label={}\n", partition.name));
    }
    
    script.push_str("exit\n");
    Ok(script)
}

#[cfg(target_os = "windows")]
fn extract_disk_number(path: &str) -> Result<u32> {
    Ok(0)
}

#[cfg(target_os = "windows")]
fn get_available_drive_letter() -> char {
    'Z'
}

#[cfg(target_os = "macos")]
async fn create_macos_partitions(device: &UsbDevice, partitions: &[PartitionConfig]) -> Result<()> {
    let output = TokioCommand::new("diskutil")
        .args(&["eraseDisk", "GPT", "TempName", &device.path])
        .output()
        .await?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to erase disk: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    for partition in partitions {
        let filesystem = match partition.filesystem.as_str() {
            "exfat" => "ExFAT",
            "fat32" => "MS-DOS FAT32",
            "hfs+" => "HFS+",
            _ => "ExFAT",
        };
        
        let output = TokioCommand::new("diskutil")
            .args(&[
                "partitionDisk",
                &device.path,
                "GPT",
                filesystem,
                &partition.name,
                &format!("{}GB", partition.size_gb)
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create partition {}: {}",
                partition.name,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    
    Ok(())
}

#[cfg(target_os = "linux")]
async fn create_linux_partitions(device: &UsbDevice, partitions: &[PartitionConfig]) -> Result<()> {
    let output = TokioCommand::new("parted")
        .args(&[&device.path, "mklabel", "gpt"])
        .output()
        .await?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to create GPT label: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let mut start_sector = 1;
    
    for partition in partitions {
        let end_sector = start_sector + (partition.size_gb * 1024 * 1024 * 1024 / 512);
        
        let output = TokioCommand::new("parted")
            .args(&[
                &device.path,
                "mkpart",
                "primary",
                &format!("{}s", start_sector),
                &format!("{}s", end_sector)
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create partition: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        start_sector = end_sector + 1;
    }
    
    for (i, partition) in partitions.iter().enumerate() {
        let partition_path = format!("{}p{}", device.path, i + 1);
        let mkfs_cmd = match partition.filesystem.as_str() {
            "exfat" => "mkfs.exfat",
            "fat32" => "mkfs.fat",
            "ext4" => "mkfs.ext4",
            _ => "mkfs.exfat",
        };
        
        let output = TokioCommand::new(mkfs_cmd)
            .args(&["-L", &partition.name, &partition_path])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to format partition {}: {}",
                partition.name,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    
    Ok(())
}

pub fn get_recommended_tesla_partitions(total_size_gb: u32) -> Vec<PartitionConfig> {
    let mut partitions = Vec::new();
    
    let dashcam_size = std::cmp::min(32, total_size_gb / 2);
    let remaining = total_size_gb - dashcam_size;
    
    partitions.push(PartitionConfig {
        name: "TeslaCam".to_string(),
        size_gb: dashcam_size,
        filesystem: "exfat".to_string(),
        purpose: "Dashcam and Sentry Mode recordings".to_string(),
    });
    
    if remaining >= 8 {
        let music_size = std::cmp::min(remaining - 4, remaining / 2);
        partitions.push(PartitionConfig {
            name: "TeslaMusic".to_string(),
            size_gb: music_size,
            filesystem: "exfat".to_string(),
            purpose: "Music files".to_string(),
        });
        
        let lightshow_size = remaining - music_size;
        if lightshow_size >= 2 {
            partitions.push(PartitionConfig {
                name: "TeslaLightshow".to_string(),
                size_gb: lightshow_size,
                filesystem: "exfat".to_string(),
                purpose: "Lightshow files".to_string(),
            });
        }
    }
    
    partitions
}