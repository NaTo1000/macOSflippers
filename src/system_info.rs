// ======================== system_info.rs ========================

use crate::helpers::pop_4u8;
use serde::Serialize;
use sysinfo::{System, MemoryRefreshKind};

#[cfg(target_os = "macos")]
use crate::gpu_info_macos::GpuInfo;

#[derive(Serialize, Debug, Clone)]
pub struct SystemInfo {
    pub cpu_usage: u8,
    pub ram_max: u16,
    pub ram_usage: u8,
    pub ram_unit: [u8; 4],
    pub gpu_usage: u8,
    pub vram_max: u16,
    pub vram_usage: u8,
    pub vram_unit: [u8; 4],
}

impl SystemInfo {
    fn get_unit(exp: u32) -> String {
        match exp {
            0 => "B",
            1 => "KB",
            2 => "MB",
            3 => "GB",
            4 => "TB",
            _ => "UB",
        }
        .to_owned()
    }

    fn get_exp(num: u64, base: u64) -> u32 {
        match num {
            x if x > u64::pow(base, 4) => 4,
            x if x > u64::pow(base, 3) => 3,
            x if x > u64::pow(base, 2) => 2,
            x if x > base => 1,
            _ => 0,
        }
    }

    pub async fn get_system_info(system: &mut System) -> Self {
        // Refresh system information
        system.refresh_cpu();
        system.refresh_memory_specifics(MemoryRefreshKind::everything());
        
        // Give CPU time to calculate usage
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        system.refresh_cpu();

        // Get CPU usage
        let cpu_usage = system.global_cpu_info().cpu_usage() as u8;

        // Get RAM information
        let ram_total = system.total_memory();
        let ram_used = system.used_memory();
        let ram_exp = Self::get_exp(ram_total, 1024);
        let ram_divisor = u64::pow(1024, ram_exp);
        
        let ram_max = (ram_total / ram_divisor) as u16;
        let ram_usage = if ram_total > 0 {
            ((ram_used as f64 / ram_total as f64) * 100.0) as u8
        } else {
            0
        };
        let ram_unit = pop_4u8(Self::get_unit(ram_exp).as_bytes());

        // Get GPU information (platform-specific)
        let (gpu_usage, vram_max, vram_usage, vram_unit) = Self::get_gpu_stats().await;

        SystemInfo {
            cpu_usage,
            ram_max,
            ram_usage,
            ram_unit,
            gpu_usage,
            vram_max,
            vram_usage,
            vram_unit,
        }
    }

    #[cfg(target_os = "macos")]
    async fn get_gpu_stats() -> (u8, u16, u8, [u8; 4]) {
        if let Some(gpu_info) = GpuInfo::get_gpu_info().await {
            let vram_exp = Self::get_exp(gpu_info.vram_max, 1024);
            let vram_divisor = u64::pow(1024, vram_exp);
            
            let vram_max = if vram_divisor > 0 {
                (gpu_info.vram_max / vram_divisor) as u16
            } else {
                0
            };
            
            let vram_usage = if gpu_info.vram_max > 0 {
                ((gpu_info.vram_used as f64 / gpu_info.vram_max as f64) * 100.0) as u8
            } else {
                0
            };
            
            let vram_unit = pop_4u8(Self::get_unit(vram_exp).as_bytes());
            let gpu_usage = gpu_info.gpu_usage as u8;

            (gpu_usage, vram_max, vram_usage, vram_unit)
        } else {
            // Fallback values if GPU info unavailable
            (0, 0, 0, pop_4u8(b"GB"))
        }
    }

    #[cfg(not(target_os = "macos"))]
    async fn get_gpu_stats() -> (u8, u16, u8, [u8; 4]) {
        // Placeholder for other platforms (Windows/Linux)
        // TODO: Implement Windows NVML/nvidia-smi parsing
        (0, 0, 0, pop_4u8(b"GB"))
    }
}
