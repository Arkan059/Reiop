use serde::{Deserialize, Serialize};
use std::fs;
use sysinfo::System;

#[derive(Serialize, Deserialize, Default)]
pub struct Metrics {
    pub ram_used_gb: f64,
    pub process_count: usize,
    pub startup_count: usize,
    pub cpu_load: f32,
    pub temp_size_mb: f64,
    pub boot_time_sec: u64,
}

pub fn collect() -> Metrics {
    let mut sys = System::new_all();
    sys.refresh_all();
    let temp_size = crate::cleanup::calculate_sizes()
        .iter()
        .find(|(n, _)| n == "User Temp")
        .map(|(_, s)| *s as f64 / 1_048_576.0)
        .unwrap_or(0.0);
    Metrics {
        ram_used_gb: sys.used_memory() as f64 / 1_073_741_824.0,
        process_count: sys.processes().len(),
        startup_count: crate::startup::scan_startup().len(),
        cpu_load: sys.global_cpu_info().cpu_usage(),
        temp_size_mb: temp_size,
        boot_time_sec: System::uptime(),
    }
}

pub fn save_baseline() -> anyhow::Result<()> {
    let m = collect();
    fs::write("baseline.json", serde_json::to_string_pretty(&m)?)?;
    Ok(())
}

pub fn compare() -> anyhow::Result<String> {
    let baseline: Metrics = fs::read_to_string("baseline.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let current = collect();
    Ok(format!(
        "RAM Used: {:.2} GB → {:.2} GB\nProcesses: {} → {}\nStartup Apps: {} → {}\nCPU Load: {:.1}% → {:.1}%\nTemp Size: {:.1} MB → {:.1} MB\nUptime: {} s → {} s",
        baseline.ram_used_gb,
        current.ram_used_gb,
        baseline.process_count,
        current.process_count,
        baseline.startup_count,
        current.startup_count,
        baseline.cpu_load,
        current.cpu_load,
        baseline.temp_size_mb,
        current.temp_size_mb,
        baseline.boot_time_sec,
        current.boot_time_sec
    ))
}