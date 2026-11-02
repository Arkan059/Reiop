use anyhow::Result;
use winreg::enums::*;
use winreg::RegKey;

pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub hive: String,
    pub path: String,
}

pub fn scan_startup() -> Vec<StartupItem> {
    let mut items = Vec::new();
    let run_paths = [
        (HKEY_CURRENT_USER, "HKCU", r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
        (HKEY_LOCAL_MACHINE, "HKLM", r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run"),
    ];
    for (hive, hive_name, path) in run_paths {
        if let Ok(key) = RegKey::predef(hive).open_subkey(path) {
            for (name, value) in key.enum_values().flatten() {
                let cmd = value.to_string();
                if !cmd.is_empty() {
                    items.push(StartupItem {
                        name,
                        command: cmd,
                        hive: hive_name.to_string(),
                        path: path.to_string(),
                    });
                }
            }
        }
    }
    items
}

pub fn disable_startup(item: &StartupItem) -> Result<()> {
    let approved_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(approved_path, KEY_SET_VALUE)?;
    let data = vec![0x03u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    key.set_raw_value(
        &item.name,
        &winreg::RegValue {
            bytes: data,
            vtype: REG_BINARY,
        },
    )?;
    Ok(())
}
