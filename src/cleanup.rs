use std::path::Path;
use std::fs;

pub fn calculate_sizes() -> Vec<(String, u64)> {
    let mut result = Vec::new();
    let targets = [
        ("User Temp", std::env::var("TEMP").unwrap_or_default()),
        ("Windows Temp", r"C:\Windows\Temp".into()),
        ("NVIDIA Shader Cache", r"C:\Users\Public\Documents\NVIDIA Corporation\NV_Cache".into()),
        ("DirectX Shader Cache", r"C:\Users\Public\Documents\DirectX\ShaderCache".into()),
        ("Windows Update Cache", r"C:\Windows\SoftwareDistribution\Download".into()),
    ];
    for (label, path) in targets {
        let size = dir_size(Path::new(&path));
        result.push((label.into(), size));
    }
    let hiber = Path::new(r"C:\hiberfil.sys");
    if hiber.exists() {
        result.push(("Hibernate File".into(), hiber.metadata().map(|m| m.len()).unwrap_or(0)));
    }
    result
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(m) = entry.metadata() {
                total += m.len();
            }
        }
    }
    total
}

pub fn clean_dir(path: &str) -> u64 {
    let p = Path::new(path);
    let mut freed = 0;
    if let Ok(entries) = fs::read_dir(p) {
        for entry in entries.flatten() {
            let ep = entry.path();
            if ep.is_dir() {
                freed += dir_size(&ep);
                let _ = fs::remove_dir_all(&ep);
            } else {
                freed += entry.metadata().map(|m| m.len()).unwrap_or(0);
                let _ = fs::remove_file(&ep);
            }
        }
    }
    freed
}