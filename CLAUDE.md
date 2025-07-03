# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

### Development
```bash
# Build and run the application
cargo run

# Build for release
cargo build --release

# Check compilation without building
cargo check

# Run tests
cargo test

# Run Tauri development mode
cargo tauri dev

# Build Tauri application for distribution
cargo tauri build
```

### Cross-Platform Builds
```bash
# Build for specific targets
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
```

### System Dependencies (Platform-Specific)
- **Ubuntu/Debian**: `sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf`
- **macOS**: `brew install create-dmg`
- **Windows**: No additional dependencies required

## Architecture Overview

This is a Tauri-based cross-platform desktop application for formatting USB drives specifically for Tesla vehicles. The architecture follows a clear separation between Rust backend logic and web frontend UI.

### Core Components

**Backend (Rust)**:
- `main.rs`: Tauri command handlers and application state management using `DeviceState` (Mutex-wrapped HashMap)
- `usb.rs`: Cross-platform USB device detection using platform-specific commands (diskutil/wmic/lsblk)
- `partitions.rs`: Partition creation logic with platform-specific implementations (diskpart/diskutil/parted)
- `tesla.rs`: Tesla-specific formatting requirements and folder structure creation

**Frontend (Web)**:
- `ui/index.html`: Modern responsive UI with device selection and configuration
- `ui/main.js`: Frontend logic using Tauri's `invoke` API to communicate with Rust backend

### Key Data Structures
- `UsbDevice`: Represents detected USB devices with name, path, size, and removable status
- `PartitionConfig`: Defines partition specifications (name, size, filesystem, purpose)
- `TeslaConfig`: Tesla-specific configuration for dashcam, music, and lightshow partitions

### Cross-Platform Implementation
The application uses conditional compilation (`#[cfg(target_os = "...")]`) to handle platform-specific USB operations:
- **Windows**: Uses `wmic` and `diskpart` commands, requires `winapi` crate
- **macOS**: Uses `diskutil` commands, requires `core-foundation` crates
- **Linux**: Uses `lsblk` and `parted` commands, requires `libc` crate

### Tesla Requirements Implementation
The tool enforces Tesla's USB requirements:
- Minimum 32GB for dashcam partition
- exFAT filesystem (preferred)
- Specific folder structure: `TeslaCam/`, `SavedClips/`, `SentryClips/`, `RecentClips/`
- Partition size validation to prevent over-allocation

### Security and Safety
- All USB operations require user confirmation
- Input validation for partition sizes
- Read-only device detection to prevent accidental formatting
- Tauri's security allowlist restricts available APIs to necessary file system and dialog operations

### State Management
Device state is managed through Tauri's state system with a `DeviceState` type that maintains detected USB devices in a thread-safe HashMap, accessible across all command handlers.

## Tesla-Specific Logic

When working with Tesla formatting features, understand that:
- Tesla requires minimum 32GB for dashcam functionality
- The tool creates specific folder structures that Tesla vehicles expect
- Partition validation ensures total size doesn't exceed device capacity
- Recommended configurations vary by USB drive size (32GB, 64-128GB, 128GB+)