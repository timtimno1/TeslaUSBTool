const { invoke } = window.__TAURI__.tauri;

let selectedDevice = null;
let devices = [];

const elements = {
    deviceList: document.getElementById('device-list'),
    refreshBtn: document.getElementById('refresh-btn'),
    formatBtn: document.getElementById('format-btn'),
    customBtn: document.getElementById('custom-btn'),
    dashcamSize: document.getElementById('dashcam-size'),
    musicSize: document.getElementById('music-size'),
    lightshowSize: document.getElementById('lightshow-size'),
    remainingSpace: document.getElementById('remaining-space'),
    progress: document.getElementById('progress'),
    progressFill: document.querySelector('.progress-fill'),
    progressText: document.querySelector('.progress-text'),
    alert: document.getElementById('alert')
};

async function refreshDevices() {
    try {
        showProgress('Scanning for USB devices...');
        elements.refreshBtn.disabled = true;
        
        devices = await invoke('get_usb_devices');
        displayDevices(devices);
        
        hideProgress();
        elements.refreshBtn.disabled = false;
        
        if (devices.length === 0) {
            showAlert('No removable USB devices found. Please connect a USB drive and try again.', 'info');
        }
    } catch (error) {
        console.error('Error refreshing devices:', error);
        showAlert('Error scanning for devices: ' + error, 'error');
        hideProgress();
        elements.refreshBtn.disabled = false;
    }
}

function displayDevices(devices) {
    if (devices.length === 0) {
        elements.deviceList.innerHTML = `
            <div class="device-item" style="text-align: center; color: #7f8c8d;">
                No USB devices found
            </div>
        `;
        return;
    }

    elements.deviceList.innerHTML = devices.map(device => `
        <div class="device-item" data-path="${device.path}" onclick="selectDevice('${device.path}')">
            <div class="device-info">
                <div>
                    <div class="device-name">${device.name}</div>
                    <div class="device-path">${device.path}</div>
                </div>
                <div class="device-size">${formatBytes(device.size)}</div>
            </div>
        </div>
    `).join('');
}

function selectDevice(path) {
    selectedDevice = devices.find(d => d.path === path);
    
    document.querySelectorAll('.device-item').forEach(item => {
        item.classList.remove('selected');
    });
    
    document.querySelector(`[data-path="${path}"]`).classList.add('selected');
    
    elements.formatBtn.disabled = false;
    elements.customBtn.disabled = false;
    
    updateRemainingSpace();
    updateRecommendedConfig();
}

function updateRemainingSpace() {
    if (!selectedDevice) return;
    
    const totalSize = selectedDevice.size / (1024 * 1024 * 1024);
    const dashcamSize = parseInt(elements.dashcamSize.value) || 0;
    const musicSize = parseInt(elements.musicSize.value) || 0;
    const lightshowSize = parseInt(elements.lightshowSize.value) || 0;
    
    const remaining = Math.max(0, totalSize - dashcamSize - musicSize - lightshowSize);
    elements.remainingSpace.value = Math.floor(remaining);
    
    const isValid = remaining >= 0;
    elements.formatBtn.disabled = !isValid;
    
    if (!isValid) {
        showAlert('Total partition size exceeds device capacity!', 'error');
    } else {
        hideAlert();
    }
}

function updateRecommendedConfig() {
    if (!selectedDevice) return;
    
    const totalSizeGB = Math.floor(selectedDevice.size / (1024 * 1024 * 1024));
    
    if (totalSizeGB < 64) {
        elements.dashcamSize.value = 32;
        elements.musicSize.value = 0;
        elements.lightshowSize.value = 0;
    } else if (totalSizeGB < 128) {
        elements.dashcamSize.value = 32;
        elements.musicSize.value = 16;
        elements.lightshowSize.value = 8;
    } else {
        elements.dashcamSize.value = 64;
        elements.musicSize.value = 32;
        elements.lightshowSize.value = 16;
    }
    
    updateRemainingSpace();
}

async function formatForTesla() {
    if (!selectedDevice) return;
    
    const config = {
        dashcam_size_gb: parseInt(elements.dashcamSize.value) || 0,
        sentry_size_gb: 0,
        music_size_gb: parseInt(elements.musicSize.value) || 0,
        lightshow_size_gb: parseInt(elements.lightshowSize.value) || 0
    };
    
    const confirmed = await showConfirmDialog(
        `Are you sure you want to format "${selectedDevice.name}"? This will erase all data on the device.`
    );
    
    if (!confirmed) return;
    
    try {
        showProgress('Formatting USB device for Tesla...');
        disableButtons();
        
        const result = await invoke('format_tesla_usb', {
            devicePath: selectedDevice.path,
            config: config
        });
        
        showAlert(result, 'success');
        hideProgress();
        enableButtons();
        
        await refreshDevices();
    } catch (error) {
        console.error('Error formatting device:', error);
        showAlert('Error formatting device: ' + error, 'error');
        hideProgress();
        enableButtons();
    }
}

async function createCustomPartitions() {
    if (!selectedDevice) return;
    
    const partitions = [
        {
            name: 'TeslaCam',
            size_gb: parseInt(elements.dashcamSize.value) || 0,
            filesystem: 'exfat',
            purpose: 'Dashcam and Sentry Mode'
        }
    ];
    
    if (parseInt(elements.musicSize.value) > 0) {
        partitions.push({
            name: 'Music',
            size_gb: parseInt(elements.musicSize.value),
            filesystem: 'exfat',
            purpose: 'Music files'
        });
    }
    
    if (parseInt(elements.lightshowSize.value) > 0) {
        partitions.push({
            name: 'LightShow',
            size_gb: parseInt(elements.lightshowSize.value),
            filesystem: 'exfat',
            purpose: 'Lightshow files'
        });
    }
    
    const confirmed = await showConfirmDialog(
        `Create ${partitions.length} partition(s) on "${selectedDevice.name}"? This will erase all data on the device.`
    );
    
    if (!confirmed) return;
    
    try {
        showProgress('Creating custom partitions...');
        disableButtons();
        
        const result = await invoke('create_custom_partitions', {
            devicePath: selectedDevice.path,
            partitions: partitions
        });
        
        showAlert(result, 'success');
        hideProgress();
        enableButtons();
        
        await refreshDevices();
    } catch (error) {
        console.error('Error creating partitions:', error);
        showAlert('Error creating partitions: ' + error, 'error');
        hideProgress();
        enableButtons();
    }
}

function showProgress(message) {
    elements.progress.style.display = 'block';
    elements.progressText.textContent = message;
    elements.progressFill.style.width = '100%';
}

function hideProgress() {
    elements.progress.style.display = 'none';
    elements.progressFill.style.width = '0%';
}

function showAlert(message, type = 'info') {
    elements.alert.textContent = message;
    elements.alert.className = `alert alert-${type}`;
    elements.alert.style.display = 'block';
    
    if (type === 'success') {
        setTimeout(() => {
            hideAlert();
        }, 5000);
    }
}

function hideAlert() {
    elements.alert.style.display = 'none';
}

function disableButtons() {
    elements.formatBtn.disabled = true;
    elements.customBtn.disabled = true;
    elements.refreshBtn.disabled = true;
}

function enableButtons() {
    elements.formatBtn.disabled = !selectedDevice;
    elements.customBtn.disabled = !selectedDevice;
    elements.refreshBtn.disabled = false;
}

function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

async function showConfirmDialog(message) {
    return confirm(message);
}

elements.refreshBtn.addEventListener('click', refreshDevices);
elements.formatBtn.addEventListener('click', formatForTesla);
elements.customBtn.addEventListener('click', createCustomPartitions);

[elements.dashcamSize, elements.musicSize, elements.lightshowSize].forEach(input => {
    input.addEventListener('input', updateRemainingSpace);
});

refreshDevices();