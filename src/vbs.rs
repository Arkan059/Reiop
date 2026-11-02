use winreg::enums::*;
use winreg::RegKey;

pub fn is_vbs_enabled() -> bool {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Control\DeviceGuard")
        .and_then(|k| k.get_value::<u32, _>("EnableVirtualizationBasedSecurity"))
        .map(|v| v == 1)
        .unwrap_or(false)
}

pub fn disable_vbs() -> anyhow::Result<()> {
    let path = r"SYSTEM\CurrentControlSet\Control\DeviceGuard";
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(path, KEY_SET_VALUE)?;
    key.set_value("EnableVirtualizationBasedSecurity", &0u32)?;

    let hvci = r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity";
    if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(hvci, KEY_SET_VALUE)
    {
        k.set_value("Enabled", &0u32)?;
    }
    Ok(())
}