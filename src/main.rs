// ======================== main.rs ========================

use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Manager, Peripheral};
use std::error::Error;
use std::time::Duration;
use sysinfo::System;

mod flipper_manager;
mod helpers;
mod system_info;

#[cfg(target_os = "macos")]
mod gpu_info_macos;

use flipper_manager::{get_central, FLIPPER_CHARACTERISTIC_UUID};
use system_info::SystemInfo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("╔═══════════════════════════════════════════╗");
    println!("║   Flipper Monitor - macOS Version        ║");
    println!("║   System Monitor via Bluetooth LE        ║");
    println!("╚═══════════════════════════════════════════╝\n");

    // Initialize Bluetooth manager
    println!("🔧 Initializing Bluetooth...");
    let manager = Manager::new().await?;
    let central = get_central(&manager).await;

    println!("✓ Bluetooth adapter ready\n");

    // Start scanning for devices
    println!("🔍 Scanning for Flipper Zero devices...");
    println!("   (Looking for devices with 'PC Mon' in name)\n");

    central.start_scan(ScanFilter::default()).await?;

    // Wait a bit for devices to be discovered
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Get list of discovered peripherals
    let peripherals = central.peripherals().await?;

    if peripherals.is_empty() {
        println!("⚠️  No Bluetooth devices found.");
        println!("   Make sure your Flipper Zero is:");
        println!("   1. Powered on");
        println!("   2. Running the PC Monitor app");
        println!("   3. In Bluetooth range\n");
        
        // Still show system info even without Flipper
        show_system_info_demo().await;
        return Ok(());
    }

    println!("📱 Found {} Bluetooth device(s)", peripherals.len());

    // Look for Flipper Zero
    let mut flipper: Option<Peripheral> = None;
    for peripheral in peripherals {
        let properties = peripheral.properties().await?;
        let local_name = properties
            .as_ref()
            .and_then(|p| p.local_name.as_ref())
            .map(|n| n.as_str())
            .unwrap_or("Unknown");

        println!("   - {}", local_name);

        if local_name.contains("PC Mon") || local_name.contains("Flipper") {
            println!("     ✓ Found Flipper Zero!");
            flipper = Some(peripheral);
            break;
        }
    }

    if let Some(flipper_device) = flipper {
        // Connect to Flipper
        println!("\n🔗 Connecting to Flipper Zero...");
        flipper_device.connect().await?;
        println!("✓ Connected!\n");

        // Discover services and characteristics
        println!("🔎 Discovering services...");
        flipper_device.discover_services().await?;
        
        let characteristics = flipper_device.characteristics();
        let flipper_char = characteristics
            .iter()
            .find(|c| c.uuid == FLIPPER_CHARACTERISTIC_UUID);

        if let Some(characteristic) = flipper_char {
            println!("✓ Found Flipper characteristic\n");
            
            // Start monitoring loop
            monitor_and_send_loop(&flipper_device, characteristic).await?;
        } else {
            println!("⚠️  Could not find Flipper characteristic UUID");
            println!("   Expected: {}\n", FLIPPER_CHARACTERISTIC_UUID);
            show_system_info_demo().await;
        }

        // Disconnect
        flipper_device.disconnect().await?;
        println!("\n👋 Disconnected from Flipper Zero");
    } else {
        println!("\n⚠️  No Flipper Zero device found with 'PC Mon' in name\n");
        show_system_info_demo().await;
    }

    Ok(())
}

/// Main monitoring loop - reads system info and sends to Flipper
async fn monitor_and_send_loop(
    peripheral: &Peripheral,
    characteristic: &Characteristic,
) -> Result<(), Box<dyn Error>> {
    println!("╔═══════════════════════════════════════════╗");
    println!("║   Starting System Monitor                ║");
    println!("║   Press Ctrl+C to stop                   ║");
    println!("╚═══════════════════════════════════════════╝\n");

    let mut sys = System::new_all();
    let mut iteration = 0;

    loop {
        iteration += 1;
        
        // Get current system information
        let info = SystemInfo::get_system_info(&mut sys).await;

        // Display to console
        println!("📊 Update #{}", iteration);
        println!("   CPU:  {}%", info.cpu_usage);
        println!("   RAM:  {} {} ({}% used)", 
            info.ram_max, 
            String::from_utf8_lossy(&info.ram_unit),
            info.ram_usage
        );
        println!("   GPU:  {}%", info.gpu_usage);
        println!("   VRAM: {} {} ({}% used)", 
            info.vram_max,
            String::from_utf8_lossy(&info.vram_unit),
            info.vram_usage
        );

        // Serialize to JSON and send to Flipper
        match serde_json::to_vec(&info) {
            Ok(data) => {
                match peripheral.write(characteristic, &data, WriteType::WithoutResponse).await {
                    Ok(_) => println!("   ✓ Sent to Flipper Zero\n"),
                    Err(e) => println!("   ⚠️  Failed to send: {}\n", e),
                }
            }
            Err(e) => println!("   ⚠️  Failed to serialize: {}\n", e),
        }

        // Wait before next update (adjust as needed)
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Show system info demo without Flipper connection
async fn show_system_info_demo() {
    println!("╔═══════════════════════════════════════════╗");
    println!("║   System Information Demo                ║");
    println!("║   (Running without Flipper connection)   ║");
    println!("╚═══════════════════════════════════════════╝\n");

    let mut sys = System::new_all();

    for i in 1..=5 {
        let info = SystemInfo::get_system_info(&mut sys).await;

        println!("📊 Reading #{}", i);
        println!("   CPU:  {}%", info.cpu_usage);
        println!("   RAM:  {} {} ({}% used)", 
            info.ram_max, 
            String::from_utf8_lossy(&info.ram_unit),
            info.ram_usage
        );
        println!("   GPU:  {}%", info.gpu_usage);
        println!("   VRAM: {} {} ({}% used)\n", 
            info.vram_max,
            String::from_utf8_lossy(&info.vram_unit),
            info.vram_usage
        );

        if i < 5 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    println!("✓ Demo complete");
}
