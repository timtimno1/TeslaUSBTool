# Tesla USB Tool

🚗 A cross-platform application built with Rust and Tauri for formatting USB drives specifically for Tesla vehicles.

## Features

- **Tesla-Optimized Formatting**: Automatically formats USB drives with the correct partitions and folder structure for Tesla Dashcam and Sentry Mode
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Custom Partitioning**: Create custom partitions for different Tesla functions:
  - Dashcam/Sentry Mode (minimum 32GB)
  - Music storage
  - Lightshow files
- **User-Friendly Interface**: Modern, responsive web-based UI
- **Automatic USB Detection**: Detects and lists removable USB devices
- **Safety Features**: Confirmation dialogs and validation to prevent accidental data loss

## Tesla USB Requirements

Based on Tesla's official documentation, this tool ensures your USB drive meets the following requirements:

- **Minimum Size**: 32GB (64GB+ recommended)
- **File System**: exFAT (recommended), FAT32, ext3, or ext4
- **Write Speed**: Minimum 4MB/s sustained write speed
- **Folder Structure**: Automatically creates required folders:
  - `TeslaCam/` - Main folder for dashcam functionality
  - `TeslaCam/SavedClips/` - Manually saved clips
  - `TeslaCam/SentryClips/` - Sentry Mode recordings
  - `TeslaCam/RecentClips/` - Recent dashcam footage

## Installation

### Download Pre-built Binaries

1. Go to the [Releases](https://github.com/yourusername/tesla-usb-tool/releases) page
2. Download the appropriate version for your operating system:
   - Windows: `tesla-usb-tool-windows-x86_64.exe`
   - macOS: `tesla-usb-tool-macos-x86_64`
   - Linux: `tesla-usb-tool-linux-x86_64`

### Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Node.js](https://nodejs.org/) (for Tauri prerequisites)

#### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf
```

**macOS:**
```bash
brew install create-dmg
```

**Windows:**
No additional dependencies required.

#### Build Steps

1. Clone the repository:
```bash
git clone https://github.com/yourusername/tesla-usb-tool.git
cd tesla-usb-tool
```

2. Build the application:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run
```

## Usage

1. **Connect your USB drive** to your computer
2. **Launch Tesla USB Tool**
3. **Click "Refresh Devices"** to scan for USB drives
4. **Select your USB drive** from the list
5. **Configure partition sizes**:
   - Dashcam: Minimum 32GB (required for Tesla)
   - Music: Optional partition for music files
   - Lightshow: Optional partition for lightshow files
6. **Click "Format for Tesla"** to start the formatting process
7. **Confirm the operation** (this will erase all data on the USB drive)
8. **Wait for completion** - the process may take several minutes

## Configuration Options

### Recommended Configurations

- **32-64GB USB**: 32GB for Dashcam only
- **64-128GB USB**: 32GB Dashcam + 16GB Music + 8GB Lightshow
- **128GB+ USB**: 64GB Dashcam + 32GB Music + 16GB Lightshow

### Custom Partitions

You can create custom partition layouts by clicking "Custom Partitions" and specifying:
- Partition names
- Sizes in GB
- File systems (exFAT recommended)
- Purpose/description

## Safety and Warnings

⚠️ **Important**: This tool will completely erase all data on the selected USB drive. Make sure to:
- Back up any important data before formatting
- Double-check you've selected the correct drive
- Ensure the USB drive is not in use by other applications

## Troubleshooting

### Common Issues

1. **USB drive not detected**
   - Ensure the drive is properly connected
   - Try a different USB port
   - Check if the drive is mounted/recognized by your OS

2. **Formatting fails**
   - Close any programs that might be using the USB drive
   - Run the application with administrator/sudo privileges
   - Check if the USB drive is write-protected

3. **Tesla doesn't recognize the drive**
   - Ensure the drive is formatted as exFAT
   - Check that the TeslaCam folder exists
   - Verify the drive has at least 32GB allocated for dashcam

### Platform-Specific Issues

**Windows**: May require running as Administrator for drive access

**macOS**: May require disk access permissions in System Preferences

**Linux**: May require sudo privileges for disk operations

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Development

### Project Structure

```
tesla-usb-tool/
├── src/
│   ├── main.rs          # Main application entry point
│   ├── usb.rs           # USB device detection and management
│   ├── partitions.rs    # Partition creation and management
│   └── tesla.rs         # Tesla-specific formatting logic
├── ui/
│   ├── index.html       # Main UI interface
│   └── main.js          # Frontend JavaScript
├── tauri.conf.json      # Tauri configuration
└── .github/workflows/   # CI/CD workflows
```

### Running Tests

```bash
cargo test
```

### Building for Different Platforms

```bash
# For your current platform
cargo build --release

# For specific targets
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This tool is not affiliated with Tesla, Inc. Use at your own risk. Always backup your data before formatting any storage device.

## Acknowledgments

- [Tauri](https://tauri.app/) for the excellent cross-platform framework
- [Tesla](https://www.tesla.com/) for the inspiration and USB requirements documentation
- The Rust community for the amazing ecosystem