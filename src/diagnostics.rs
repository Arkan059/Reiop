use chrono::Datelike;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: bool,
    pub weight: u32,
    pub tip: String,
}

pub fn run_checks(info: &crate::sysinfo::SystemInfo) -> Vec<CheckResult> {
    let mut checks = Vec::new();

    checks.push(CheckResult {
        name: "Power Plan".into(),
        status: crate::tweaks::is_high_performance_active(),
        weight: 12,
        tip: "Activate High Performance or Ultimate Performance plan.".into(),
    });

    checks.push(CheckResult {
        name: "Game Mode".into(),
        status: crate::tweaks::game_mode_enabled(),
        weight: 6,
        tip: "Enable Windows Game Mode.".into(),
    });

    checks.push(CheckResult {
        name: "HAGS".into(),
        status: crate::tweaks::hags_enabled(),
        weight: 10,
        tip: "Enable Hardware-Accelerated GPU Scheduling.".into(),
    });

    checks.push(CheckResult {
        name: "Dual-Channel RAM".into(),
        status: info.is_dual_channel,
        weight: 8,
        tip: "Install an additional stick of RAM for dual-channel performance.".into(),
    });

    checks.push(CheckResult {
        name: "System Disk Type".into(),
        status: info.disk_type == "SSD",
        weight: 10,
        tip: "Use an SSD for the operating system.".into(),
    });

    checks.push(CheckResult {
        name: "Refresh Rate".into(),
        status: info.refresh_rate >= 100,
        weight: 8,
        tip: "Set monitor to its maximum refresh rate.".into(),
    });

    checks.push(CheckResult {
        name: "VBS / Core Isolation".into(),
        status: !crate::vbs::is_vbs_enabled(),
        weight: 8,
        tip: "Disable VBS for competitive gaming if security policy allows.".into(),
    });

    checks.push(CheckResult {
        name: "RAM Capacity".into(),
        status: info.ram_gb >= 16.0,
        weight: 8,
        tip: "Upgrade to at least 16 GB of RAM for modern gaming.".into(),
    });

    checks.push(CheckResult {
        name: "GPU VRAM".into(),
        status: info.vram_gb >= 6.0,
        weight: 8,
        tip: "Modern games require at least 6 GB of VRAM.".into(),
    });

    let current_year = chrono::Local::now().year();
    let driver_ok = match &info.gpu_driver_date {
        Some(d) => {
            if let Ok(year) = d.get(0..4).unwrap_or("0").parse::<i32>() {
                (current_year - year) <= 1
            } else { false }
        },
        None => false,
    };
    checks.push(CheckResult {
        name: "GPU Driver Age".into(),
        status: driver_ok,
        weight: 8,
        tip: "Update your GPU drivers to the latest version.".into(),
    });

    checks.push(CheckResult {
        name: "CPU Threads".into(),
        status: info.cpu_threads >= 8,
        weight: 7,
        tip: "A CPU with 8+ threads improves gaming stability.".into(),
    });

    let heat_ok = match info.cpu_temp {
        Some(t) => t < 85.0,
        None => true,
    };
    checks.push(CheckResult {
        name: "CPU Temperature".into(),
        status: heat_ok,
        weight: 7,
        tip: "Ensure your CPU cooling is adequate (under 85?C).".into(),
    });

    checks
}

pub fn calculate_score(checks: &[CheckResult]) -> (u32, &'static str) {
    let total: u32 = checks.iter().map(|c| c.weight).sum();
    let earned: u32 = checks.iter().filter(|c| c.status).map(|c| c.weight).sum();
    let score = if total == 0 { 0 } else { earned * 100 / total };

    let verdict = if score >= 85 {
        "Excellent"
    } else if score >= 70 {
        "Good"
    } else if score >= 50 {
        "Fair"
    } else {
        "Needs Improvement"
    };
    (score, verdict)
}
