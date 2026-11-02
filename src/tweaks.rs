use crate::backup::BackupManager;
use anyhow::Result;
use winreg::enums::*;
use winreg::{RegKey, HKEY};

fn snap(hive: HKEY, path: &str, name: &str, bk: &mut BackupManager) -> Result<RegKey> {
    let key = RegKey::predef(hive).open_subkey_with_flags(path, KEY_SET_VALUE)?;
    let old = key.get_raw_value(name).ok();
    let data = old
        .as_ref()
        .map(|v| serde_json::json!(v.bytes.iter().map(|b| *b as u64).collect::<Vec<_>>()));
    bk.record_registry(
        if hive == HKEY_CURRENT_USER { "HKCU" } else { "HKLM" },
        path,
        name,
        data,
        None,
    );
    Ok(key)
}

fn u32w(hive: HKEY, path: &str, name: &str, v: u32, bk: &mut BackupManager) -> Result<()> {
    snap(hive, path, name, bk)?.set_value(name, &v)?;
    Ok(())
}

fn strw(hive: HKEY, path: &str, name: &str, v: &str, bk: &mut BackupManager) -> Result<()> {
    snap(hive, path, name, bk)?.set_value(name, &v.to_string())?;
    Ok(())
}

pub fn apply_tweak(name: &str, bk: &mut BackupManager) -> Result<()> {
    match name {
        "power_plan" => activate_ultimate(bk)?,
        "power_throttling" => u32w(HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling", "PowerThrottlingOff", 1, bk)?,
        "game_mode" => u32w(HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\GameBar", "AllowAutoGameMode", 1, bk)?,
        "game_dvr" => {
            u32w(HKEY_CURRENT_USER, r"System\GameConfigStore", "GameDVR_Enabled", 0, bk)?;
            u32w(HKEY_CURRENT_USER, r"System\GameConfigStore", "GameDVR_FSEBehavior", 2, bk)?;
        }
        "mouse_accel" => {
            strw(HKEY_CURRENT_USER, r"Control Panel\Mouse", "MouseSpeed", "0", bk)?;
            strw(HKEY_CURRENT_USER, r"Control Panel\Mouse", "MouseThreshold1", "0", bk)?;
            strw(HKEY_CURRENT_USER, r"Control Panel\Mouse", "MouseThreshold2", "0", bk)?;
        }
        "hags" => u32w(HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers", "HwSchMode", 2, bk)?,
        "windowed_opt" => {
            u32w(HKEY_CURRENT_USER, r"Software\Microsoft\DirectX\UserGpuPreferences", "SwapEffectUpgradeEnable", 1, bk)?;
            u32w(HKEY_CURRENT_USER, r"Software\Microsoft\DirectX\UserGpuPreferences", "VRROptimizeEnable", 1, bk)?;
        }
        "delivery_opt" => u32w(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\DeliveryOptimization\Config", "DODownloadMode", 0, bk)?,
        "update_hours" => {
            u32w(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\WindowsUpdate\UX\Settings", "ActiveHoursStart", 8, bk)?;
            u32w(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\WindowsUpdate\UX\Settings", "ActiveHoursEnd", 23, bk)?;
        }
        "sticky_keys" => strw(HKEY_CURRENT_USER, r"Control Panel\Accessibility\StickyKeys", "Flags", "506", bk)?,
        "tips_ads" => {
            u32w(HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SubscribedContent-338389Enabled", 0, bk)?;
            u32w(HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SystemPaneSuggestionsEnabled", 0, bk)?;
            u32w(HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo", "Enabled", 0, bk)?;
        }
        "visual_effects" => {
            u32w(HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects", "VisualFXSetting", 2, bk)?;
            u32w(HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "TaskbarAnimations", 0, bk)?;
        }
        "background_apps" => u32w(HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications", "GlobalUserDisabled", 1, bk)?,
        "startup_delay" => {
            let p = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize";
            let key = RegKey::predef(HKEY_CURRENT_USER).create_subkey(p)?.0;
            bk.record_registry("HKCU", p, "StartupDelayInMSec", None, None);
            key.set_value("StartupDelayInMSec", &0u32)?;
        }
        "priority_sep" => u32w(HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Control\PriorityControl", "Win32PrioritySeparation", 38, bk)?,
        "mmcss" => {
            u32w(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", "SystemResponsiveness", 10, bk)?;
            u32w(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", "NetworkThrottlingIndex", 0xffffffff, bk)?;
            u32w(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games", "GPU Priority", 8, bk)?;
            u32w(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games", "Priority", 6, bk)?;
            strw(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games", "Scheduling Category", "High", bk)?;
        }
        "diagtrack" => {
            u32w(HKEY_LOCAL_MACHINE, r"SYSTEM\CurrentControlSet\Services\DiagTrack", "Start", 4, bk)?;
            let _ = std::process::Command::new("sc").args(["stop", "DiagTrack"]).output();
        }
        _ => {}
    }
    Ok(())
}

fn activate_ultimate(bk: &mut BackupManager) -> Result<()> {
    use windows::core::GUID;
    use windows::Win32::System::Power::*;
    let ult = GUID::from_u128(0x8c5e7fda_e8bf_4a96_9a85_a6e23a8c635c);
    unsafe {
        let mut a: *mut GUID = std::ptr::null_mut();
        let _ = PowerGetActiveScheme(None, &mut a);
        if !a.is_null() {
            let bytes: [u8; 16] = std::ptr::read(a as *const [u8; 16]);
            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
            bk.record_power_scheme(&hex);
        }
        let _ = PowerSetActiveScheme(None, Some(&ult));
    }
    Ok(())
}

pub fn is_high_performance_active() -> bool {
    use windows::core::GUID;
    use windows::Win32::System::Power::*;
    unsafe {
        let mut a: *mut GUID = std::ptr::null_mut();
        let _ = PowerGetActiveScheme(None, &mut a);
        if a.is_null() {
            return false;
        }
        let bytes: [u8; 16] = std::ptr::read(a as *const [u8; 16]);
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        hex.starts_with("da7f5e8c")
    }
}

pub fn activate_power_scheme_by_guid(hex: &str) -> Result<()> {
    use windows::core::GUID;
    use windows::Win32::System::Power::*;
    if hex.len() != 32 {
        return Ok(());
    }
    let mut bytes = [0u8; 16];
    for i in 0..16 {
        bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)?;
    }
    let guid: GUID = unsafe { std::mem::transmute(bytes) };
    unsafe {
        let _ = PowerSetActiveScheme(None, Some(&guid));
    }
    Ok(())
}

pub fn game_mode_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"SOFTWARE\Microsoft\GameBar")
        .and_then(|k| k.get_value::<u32, _>("AllowAutoGameMode"))
        .map(|v| v == 1)
        .unwrap_or(false)
}

pub fn hags_enabled() -> bool {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers")
        .and_then(|k| k.get_value::<u32, _>("HwSchMode"))
        .map(|v| v == 2)
        .unwrap_or(false)
}
