// ======================== main.rs ========================

use btleplug::api::{Central, CentralEvent, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::error::Error;

mod flipper_manager;
mod helpers;
mod system_info;

#[cfg(target_os = "macos")]
mod gpu_info_macos;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Flipper Monitor - macOS Version");
    println!("================================");
    
    // Initialize system info
    let mut sys = sysinfo::System::new_all();
    
    // Get system information
    let info = system_info::SystemInfo::get_system_info(&mut sys).await;
    
    println!("\nSystem Information:");
    println!("CPU Usage: {}%", info.cpu_usage);
    println!("RAM: {} {} ({}% used)", 
        info.ram_max, 
        String::from_utf8_lossy(&info.ram_unit),
        info.ram_usage
    );
    println!("GPU Usage: {}%", info.gpu_usage);
    println!("VRAM: {} {} ({}% used)", 
        info.vram_max,
        String::from_utf8_lossy(&info.vram_unit),
        info.vram_usage
    );
    
    // TODO: Add BLE scanning and Flipper connection logic
    println!("\nBluetooth scanning not yet implemented in this demo");
    
    Ok(())
}
