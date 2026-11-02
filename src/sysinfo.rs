use serde::{Deserialize, Serialize};
use sysinfo::System;
use wmi::{COMLibrary, WMIConnection};
use windows::Win32::Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS};

#[derive(Serialize, Default)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub build: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub ram_gb: f64,
    pub ram_type: String,
    pub is_dual_channel: bool,
    pub gpu_name: String,
    pub gpu_vendor: String,
    pub vram_gb: f64,
    pub gpu_driver_date: Option<String>,
    pub cpu_temp: Option<f32>,
    pub disk_type: String,
    pub refresh_rate: u32,
}

#[derive(Deserialize)]
struct GpuRow {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "AdapterRAM")]
    adapter_ram: Option<u32>,
    #[serde(rename = "DriverDate")]
    driver_date: Option<String>,
}

#[derive(Deserialize)]
struct RamRow {
    #[serde(rename = "SMBIOSMemoryType")]
    smbios_memory_type: Option<u32>,
}

#[derive(Deserialize)]
struct DiskRow {
    #[serde(rename = "MediaType")]
    media_type: Option<u16>,
}

#[derive(Deserialize)]
struct ThermalRow {
    #[serde(rename = "CurrentTemperature")]
    current_temperature: Option<u32>,
}

pub fn collect() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_default();
    let os_version = System::kernel_version().unwrap_or_default();
    let build = System::os_version().unwrap_or_default();

    let cpu_cores = sys.physical_core_count().unwrap_or(0);
    let cpu_threads = sys.cpus().len();
    
    let raw_ram = sys.total_memory() as f64 / 1_073_741_824.0;
    let standard_sizes = [2.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 96.0, 128.0];
    let mut ram_gb = raw_ram.round();
    for &size in standard_sizes.iter() {
        if raw_ram > size - 2.5 && raw_ram <= size + 0.5 {
            ram_gb = size;
            break;
        }
    }

    let refresh_rate = get_refresh_rate();

    let (gpu_name, gpu_vendor, vram_gb, gpu_driver_date) = query_gpu_wmi();
    let (ram_type, is_dual_channel) = query_ram_info();
    let disk_type = query_disk_type();
    let cpu_temp = query_cpu_temp();

    SystemInfo {
        os_name,
        os_version,
        build,
        cpu_cores,
        cpu_threads,
        ram_gb,
        ram_type,
        is_dual_channel,
        gpu_name,
        gpu_vendor,
        vram_gb,
        gpu_driver_date,
        cpu_temp,
        disk_type,
        refresh_rate,
    }
}

fn get_refresh_rate() -> u32 {
    unsafe {
        let mut dm = DEVMODEW::default();
        dm.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        if EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &mut dm).as_bool() {
            if dm.dmDisplayFrequency > 1 {
                return dm.dmDisplayFrequency;
            }
        }
    }
    60
}

fn query_gpu_wmi() -> (String, String, f64, Option<String>) {
    let com = match COMLibrary::new() {
        Ok(c) => c,
        Err(_) => return ("Unknown".into(), "Unknown".into(), 0.0, None),
    };
    let conn = match WMIConnection::new(com) {
        Ok(c) => c,
        Err(_) => return ("Unknown".into(), "Unknown".into(), 0.0, None),
    };
    let q = "SELECT Name, AdapterRAM, DriverDate FROM Win32_VideoController";
    let rows: Vec<GpuRow> = match conn.raw_query(q) {
        Ok(r) => r,
        Err(_) => return ("Unknown".into(), "Unknown".into(), 0.0, None),
    };
    let row = match rows.into_iter().next() {
        Some(r) => r,
        None => return ("Unknown".into(), "Unknown".into(), 0.0, None),
    };
    let name = row.name.unwrap_or_default();
    let vendor = if name.contains("NVIDIA") {
        "NVIDIA"
    } else if name.contains("AMD") || name.contains("Radeon") {
        "AMD"
    } else {
        "Intel"
    };
    let vram = row.adapter_ram.unwrap_or(0) as f64 / 1_073_741_824.0;
    (name, vendor.to_string(), vram, row.driver_date)
}

fn query_ram_info() -> (String, bool) {
    let com = match COMLibrary::new() {
        Ok(c) => c,
        Err(_) => return ("Unknown".into(), false),
    };
    let conn = match WMIConnection::new(com) {
        Ok(c) => c,
        Err(_) => return ("Unknown".into(), false),
    };
    let q = "SELECT SMBIOSMemoryType FROM Win32_PhysicalMemory";
    let rows: Vec<RamRow> = match conn.raw_query(q) {
        Ok(r) => r,
        Err(_) => return ("Unknown".into(), false),
    };
    
    let count = rows.len();
    let dual_channel = count >= 2;
    
    let ram_type = match rows.first().and_then(|r| r.smbios_memory_type) {
        Some(26) => "DDR4".into(),
        Some(34) => "DDR5".into(),
        Some(24) => "DDR3".into(),
        _ => "Unknown".into(),
    };
    
    (ram_type, dual_channel)
}

fn query_disk_type() -> String {
    let com = match COMLibrary::new() {
        Ok(c) => c,
        Err(_) => return "Unknown".into(),
    };
    let ns = "ROOT\\Microsoft\\Windows\\Storage";
    let conn = match WMIConnection::with_namespace_path(ns, com) {
        Ok(c) => c,
        Err(_) => return "Unknown".into(),
    };
    let q = "SELECT MediaType FROM MSFT_PhysicalDisk";
    let rows: Vec<DiskRow> = match conn.raw_query(q) {
        Ok(r) => r,
        Err(_) => return "Unknown".into(),
    };
    
    for row in rows {
        match row.media_type {
            Some(4) => return "SSD".into(),
            Some(3) => return "HDD".into(),
            _ => {}
        }
    }
    "Unknown".into()
}

fn query_cpu_temp() -> Option<f32> {
    let com = COMLibrary::new().ok()?;
    let conn = WMIConnection::new(com).ok()?;
    let q = "SELECT CurrentTemperature FROM Win32_PerfFormattedData_Counters_ThermalZoneInformation";
    let rows: Vec<ThermalRow> = conn.raw_query(q).ok()?;
    if let Some(row) = rows.first() {
        if let Some(t) = row.current_temperature {
            if t > 2000 { 
                return Some((t as f32 / 10.0) - 273.15);
            }
        }
    }
    None
}
