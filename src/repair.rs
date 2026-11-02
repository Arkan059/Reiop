use std::process::Command;

pub fn run_dism() -> bool {
    let status = Command::new("DISM")
        .args(["/Online", "/Cleanup-Image", "/RestoreHealth"])
        .status();
    status.map(|s| s.success()).unwrap_or(false)
}

pub fn run_sfc() -> bool {
    let status = Command::new("sfc").arg("/scannow").status();
    status.map(|s| s.success()).unwrap_or(false)
}