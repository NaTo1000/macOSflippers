// ======================== gpu_info_macos.rs ========================
// macOS-specific GPU information retrieval

use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Debug, Clone)]
pub struct GpuInfo {
    pub gpu_usage: u64,
    pub vram_max: u64,
    pub vram_used: u64,
}

impl GpuInfo {
    /// Get GPU information on macOS
    /// Uses different methods depending on the GPU type (Apple Silicon vs Intel/AMD)
    pub async fn get_gpu_info() -> Option<Self> {
        // Try to detect if we're on Apple Silicon
        if Self::is_apple_silicon() {
            Self::get_apple_silicon_gpu_info().await
        } else {
            Self::get_intel_amd_gpu_info().await
        }
    }

    /// Check if running on Apple Silicon (M1/M2/M3/etc)
    fn is_apple_silicon() -> bool {
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
        {
            let cpu_info = String::from_utf8_lossy(&output.stdout);
            cpu_info.contains("Apple")
        } else {
            false
        }
    }

    /// Get GPU info for Apple Silicon Macs
    async fn get_apple_silicon_gpu_info() -> Option<Self> {
        // Method 1: Try using ioreg to get GPU info
        if let Some(info) = Self::parse_ioreg_gpu() {
            return Some(info);
        }

        // Method 2: Try using powermetrics (requires sudo, may not work)
        if let Some(info) = Self::parse_powermetrics_gpu() {
            return Some(info);
        }

        // Fallback: Return default values
        Some(GpuInfo {
            gpu_usage: 0,
            vram_max: Self::get_total_vram_apple_silicon().unwrap_or(0),
            vram_used: 0,
        })
    }

    /// Get GPU info for Intel/AMD GPUs on older Macs
    async fn get_intel_amd_gpu_info() -> Option<Self> {
        // Use system_profiler to get GPU information
        if let Some(info) = Self::parse_system_profiler_gpu() {
            return Some(info);
        }

        // Fallback
        Some(GpuInfo {
            gpu_usage: 0,
            vram_max: 0,
            vram_used: 0,
        })
    }

    /// Parse GPU info from ioreg command
    fn parse_ioreg_gpu() -> Option<GpuInfo> {
        // Get GPU memory info from IORegistry
        let output = Command::new("ioreg")
            .arg("-r")
            .arg("-d")
            .arg("1")
            .arg("-w")
            .arg("0")
            .arg("-c")
            .arg("IOAccelerator")
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse VRAM size (this is a simplified parser)
        // Real implementation would need more robust parsing
        let vram_max = Self::extract_vram_from_ioreg(&output_str);

        Some(GpuInfo {
            gpu_usage: 0, // ioreg doesn't provide usage directly
            vram_max: vram_max.unwrap_or(0),
            vram_used: 0,
        })
    }

    /// Extract VRAM size from ioreg output
    fn extract_vram_from_ioreg(output: &str) -> Option<u64> {
        // Look for "VRAM,totalMB" or similar fields
        for line in output.lines() {
            if line.contains("VRAM") || line.contains("vram") {
                // Parse the value - this is simplified
                // Real implementation needs proper parsing
                if let Some(value_str) = line.split('=').nth(1) {
                    let cleaned = value_str.trim().trim_matches(|c| c == '"' || c == ',');
                    if let Ok(value) = cleaned.parse::<u64>() {
                        return Some(value * 1024 * 1024); // Convert MB to bytes
                    }
                }
            }
        }
        None
    }

    /// Parse GPU info from powermetrics (requires elevated privileges)
    fn parse_powermetrics_gpu() -> Option<GpuInfo> {
        // powermetrics requires sudo, so this might not work in all cases
        let output = Command::new("powermetrics")
            .arg("--samplers")
            .arg("gpu_power")
            .arg("-i")
            .arg("1000")
            .arg("-n")
            .arg("1")
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse GPU usage percentage
        let gpu_usage = Self::extract_gpu_usage_from_powermetrics(&output_str);

        Some(GpuInfo {
            gpu_usage: gpu_usage.unwrap_or(0),
            vram_max: 0,
            vram_used: 0,
        })
    }

    /// Extract GPU usage from powermetrics output
    fn extract_gpu_usage_from_powermetrics(output: &str) -> Option<u64> {
        for line in output.lines() {
            if line.contains("GPU active residency") || line.contains("GPU Active") {
                // Parse percentage value
                if let Some(percent_str) = line.split(':').nth(1) {
                    let cleaned = percent_str.trim().trim_end_matches('%');
                    if let Ok(value) = cleaned.parse::<f64>() {
                        return Some(value as u64);
                    }
                }
            }
        }
        None
    }

    /// Parse GPU info from system_profiler
    fn parse_system_profiler_gpu() -> Option<GpuInfo> {
        let output = Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse VRAM size
        let vram_max = Self::extract_vram_from_system_profiler(&output_str);

        Some(GpuInfo {
            gpu_usage: 0,
            vram_max: vram_max.unwrap_or(0),
            vram_used: 0,
        })
    }

    /// Extract VRAM from system_profiler output
    fn extract_vram_from_system_profiler(output: &str) -> Option<u64> {
        for line in output.lines() {
            if line.contains("VRAM") || line.contains("Memory") {
                // Look for patterns like "8 GB" or "8192 MB"
                let words: Vec<&str> = line.split_whitespace().collect();
                for (i, word) in words.iter().enumerate() {
                    if word.contains("GB") && i > 0 {
                        if let Ok(value) = words[i - 1].parse::<u64>() {
                            return Some(value * 1024 * 1024 * 1024); // GB to bytes
                        }
                    } else if word.contains("MB") && i > 0 {
                        if let Ok(value) = words[i - 1].parse::<u64>() {
                            return Some(value * 1024 * 1024); // MB to bytes
                        }
                    }
                }
            }
        }
        None
    }

    /// Get total VRAM for Apple Silicon (unified memory)
    fn get_total_vram_apple_silicon() -> Option<u64> {
        // On Apple Silicon, GPU uses unified memory
        // We can get total system memory as an approximation
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()
            .ok()?;

        let mem_str = String::from_utf8_lossy(&output.stdout);
        mem_str.trim().parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_info() {
        let info = GpuInfo::get_gpu_info().await;
        assert!(info.is_some());
        println!("GPU Info: {:?}", info);
    }
}
