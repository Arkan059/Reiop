use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

pub fn is_elevated() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}

pub fn ensure_admin() {
    if is_elevated() {
        return;
    }
    crate::ui::warn("Administrator privileges required. Requesting elevation...");
    if relaunch_as_admin() {
        std::process::exit(0);
    } else {
        crate::ui::error("Elevation cancelled. Right-click the exe and select Run as administrator.");
        std::process::exit(1);
    }
}

fn relaunch_as_admin() -> bool {
    let exe = std::env::current_exe().unwrap_or_default();
    let exe_w: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let verb_w: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();

    let result = unsafe {
        ShellExecuteW(
            HWND(std::ptr::null_mut()),
            PCWSTR(verb_w.as_ptr()),
            PCWSTR(exe_w.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    (result.0 as isize) > 32
}
