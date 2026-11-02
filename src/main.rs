use std::io::{self, Write};
use colored::Colorize;

mod admin;
mod backup;
mod cleanup;
mod diagnostics;
mod metrics;
mod report;
mod repair;
mod startup;
mod sysinfo;
mod tweaks;
mod ui;
mod vbs;

fn enable_vt_and_colors() {
    use windows::Win32::System::Console::*;
    unsafe {
        if let Ok(handle) = GetStdHandle(STD_OUTPUT_HANDLE) {
            let mut mode = CONSOLE_MODE::default();
            if GetConsoleMode(handle, &mut mode).is_ok() {
                let _ = SetConsoleMode(handle, CONSOLE_MODE(mode.0 | 0x0004));
            }
        }
    }
    colored::control::set_override(true);
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H\x1B[3J");
    io::stdout().flush().ok();
}

fn pause() {
    print!("\nPress Enter to continue...");
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
}

fn multi_select(title: &str, items: &[&str]) -> Vec<usize> {
    println!();
    ui::info(title);
    for (i, item) in items.iter().enumerate() {
        println!("  [{}] {}", i + 1, item);
    }
    loop {
        print!("Enter numbers (e.g. 1,3,5), 'all', or 'none': ");
        io::stdout().flush().ok();
        let mut s = String::new();
        if io::stdin().read_line(&mut s).is_err() {
            continue;
        }
        let trimmed = s.trim().to_lowercase();
        if trimmed == "all" {
            return (0..items.len()).collect();
        }
        if trimmed == "none" {
            return vec![];
        }
        let mut result = Vec::new();
        let mut valid = true;
        for part in trimmed.split(',') {
            let p = part.trim();
            if p.is_empty() {
                continue;
            }
            match p.parse::<usize>() {
                Ok(n) if n >= 1 && n <= items.len() => result.push(n - 1),
                _ => {
                    valid = false;
                    break;
                }
            }
        }
        if valid && !result.is_empty() {
            return result;
        }
        ui::error("Invalid input. Try again.");
    }
}

fn main() {
    enable_vt_and_colors();
    admin::ensure_admin();
    
    let info = sysinfo::collect();

    let mut backup = backup::BackupManager::new();

    loop {
        clear_screen();
        ui::print_logo();
        
        println!("{}", "Discord & Instagram: arkanquixote".cyan().bold());
        println!();
        println!("{}", format!("CPU: {} | GPU: {} | RAM: {:.0} GB", 
            if info.cpu_threads > 0 { format!("{} Cores / {} Threads", info.cpu_cores, info.cpu_threads) } else { "Unknown".to_string() },
            if info.gpu_name.is_empty() { "Unknown" } else { &info.gpu_name },
            info.ram_gb
        ).white());

        println!("\n{}", "Main Menu".cyan().bold());
        println!("  [1] {} (Detailed system analysis)", "Diagnostic & HTML Report".cyan());
        println!("  [2] {} (Safe performance tweaks)", "Tweak Profiles & Custom Optimization".green());
        println!("  [3] {} (Manage startup apps)", "Startup Applications Manager".white());
        println!("  [4] {} (Free up disk space)", "Cleanup Center".white());
        println!("  [5] {} (Fix system files)", "Windows Repair (DISM / SFC)".white());
        println!("  [6] {} (Optimize for gaming)", "Toggle VBS / Core Isolation".yellow());
        println!("  [7] {} (Compare performance)", "Before / After Metrics".cyan());
        println!("  [8] {} (Revert all changes)", "Full Undo (Restore Backup)".red());
        println!("  [0] Exit");
        
        println!("\n{}", "Every change is backed up and can be undone. The tool does not collect any data from you.".yellow().italic());

        let mut s = String::new();
        print!("\nSelection (0-8): ");
        io::stdout().flush().ok();
        io::stdin().read_line(&mut s).ok();
        
        let choice = match s.trim().parse::<usize>() {
            Ok(0) => 8,
            Ok(n) if n >= 1 && n <= 8 => n - 1,
            _ => { ui::error("Invalid selection."); pause(); continue; }
        };

        match choice {
            0 => diagnostic_menu(),
            1 => tweak_menu(&mut backup),
            2 => startup_menu(),
            3 => cleanup_menu(),
            4 => repair_menu(),
            5 => vbs_menu(),
            6 => metrics_menu(),
            7 => full_undo(&backup),
            8 => break,
            _ => {}
        }
    }
}

fn diagnostic_menu() {
    clear_screen();
    ui::info("Collecting system information...");
    let info = sysinfo::collect();
    let checks = diagnostics::run_checks(&info);
    let (score, verdict) = diagnostics::calculate_score(&checks);

    ui::success(&format!("Score: {}/100 - {}", score, verdict));
    for c in &checks {
        let status = if c.status { "PASS" } else { "FAIL" };
        let line = format!("{} - {} ({})", c.name, status, c.tip);
        if c.status { ui::success(&line) } else { ui::warn(&line) }
    }
    if let Err(e) = report::generate(&info, &checks, score, verdict) {
        ui::error(&format!("Report error: {}", e));
    } else {
        ui::success("Report saved to Reiop_Report.html");
    }
    pause();
}

fn tweak_menu(backup: &mut backup::BackupManager) {
    clear_screen();
    println!();
    ui::info("Select Profile");
    println!("  [1] Balanced");
    println!("  [2] Competitive");
    println!("  [3] Custom");
    println!("  [0] Back");
    print!("\nSelection (0-3): ");
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    let profile = match s.trim().parse::<usize>() {
        Ok(0) => return,
        Ok(n) if n >= 1 && n <= 3 => n - 1,
        _ => return,
    };

    let tweaks = vec![
        ("Ultimate Performance Power Plan", "power_plan"),
        ("Disable Power Throttling", "power_throttling"),
        ("Disable Game DVR", "game_dvr"),
        ("Disable Mouse Acceleration", "mouse_accel"),
        ("Enable HAGS", "hags"),
        ("Windowed Optimizations", "windowed_opt"),
        ("Disable Delivery Optimization", "delivery_opt"),
        ("Restrict Update Hours", "update_hours"),
        ("Disable Sticky Keys", "sticky_keys"),
        ("Disable Tips & Ads", "tips_ads"),
        ("Reduce Visual Effects", "visual_effects"),
        ("Restrict Background Apps", "background_apps"),
        ("Remove Startup Delay", "startup_delay"),
        ("Foreground Priority (38)", "priority_sep"),
        ("MMCSS Gaming Profile", "mmcss"),
        ("Disable DiagTrack", "diagtrack"),
    ];

    let selected: Vec<usize> = match profile {
        0 | 1 => (0..tweaks.len()).collect(),
        2 => {
            let labels: Vec<&str> = tweaks.iter().map(|(n, _)| *n).collect();
            multi_select("Select Tweaks", &labels)
        }
        _ => return,
    };

    clear_screen();
    for &i in &selected {
        let (label, key) = tweaks[i];
        ui::info(&format!("Applying: {}", label));
        match tweaks::apply_tweak(key, backup) {
            Ok(_) => ui::success(&format!("OK: {}", label)),
            Err(e) => ui::error(&format!("FAIL: {} - {}", label, e)),
        }
    }
    if let Err(e) = backup.save() {
        ui::error(&format!("Backup save failed: {}", e));
    } else {
        ui::success("Backup saved to backup.json");
    }
    pause();
}

fn startup_menu() {
    clear_screen();
    let items = startup::scan_startup();
    if items.is_empty() {
        ui::info("No startup items found.");
        pause();
        return;
    }
    println!();
    ui::info("Startup Items");
    for (i, item) in items.iter().enumerate() {
        println!("  [{}] {} - {}", i + 1, item.name, item.command);
    }
    print!("Enter index to disable (0 to cancel): ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    if let Ok(idx) = input.trim().parse::<usize>() {
        if idx > 0 && idx <= items.len() {
            if startup::disable_startup(&items[idx - 1]).is_ok() {
                ui::success("Startup item disabled.");
            } else {
                ui::error("Failed to disable.");
            }
        }
    }
    pause();
}

fn cleanup_menu() {
    clear_screen();
    let sizes = cleanup::calculate_sizes();
    println!();
    ui::info("Cleanup Sizes");
    for (label, size) in &sizes {
        println!("  {}: {:.2} MB", label, *size as f64 / 1_048_576.0);
    }
    let options = vec![
        "Clean User Temp",
        "Clean Windows Temp",
        "Clean Shader Caches",
        "Clean Windows Update Cache",
        "Cancel",
    ];
    println!();
    ui::info("Cleanup Options");
    for (i, opt) in options.iter().enumerate() {
        println!("  [{}] {}", i + 1, opt);
    }
    print!("\nSelection (1-5): ");
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    let choice = match s.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= 5 => n - 1,
        _ => return,
    };

    let paths = [
        std::env::var("TEMP").unwrap_or_default(),
        r"C:\Windows\Temp".into(),
        r"C:\Users\Public\Documents\NVIDIA Corporation\NV_Cache".into(),
        r"C:\Windows\SoftwareDistribution\Download".into(),
    ];
    if choice < 4 {
        ui::destructive("Are you sure? This cannot be undone.");
        let confirm = menu_confirm();
        if confirm {
            let freed = cleanup::clean_dir(&paths[choice]);
            ui::success(&format!("Freed {:.2} MB", freed as f64 / 1_048_576.0));
        }
    }
    pause();
}

fn menu_confirm() -> bool {
    print!("Confirm (y/n): ");
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    s.trim().to_lowercase() == "y"
}

fn repair_menu() {
    clear_screen();
    ui::warn("Running DISM... This may take several minutes.");
    if repair::run_dism() {
        ui::success("DISM completed successfully.");
    } else {
        ui::error("DISM failed.");
    }
    ui::warn("Running SFC...");
    if repair::run_sfc() {
        ui::success("SFC completed successfully.");
    } else {
        ui::error("SFC failed.");
    }
    pause();
}

fn vbs_menu() {
    clear_screen();
    let enabled = vbs::is_vbs_enabled();
    ui::info(&format!("VBS enabled: {}", enabled));
    if enabled {
        ui::destructive("Disabling VBS may reduce security. Proceed?");
        let confirm = menu_confirm();
        if confirm {
            if vbs::disable_vbs().is_ok() {
                ui::success("VBS disabled. Restart required.");
            } else {
                ui::error("Failed to disable VBS.");
            }
        }
    } else {
        ui::info("VBS is already disabled.");
    }
    pause();
}

fn metrics_menu() {
    clear_screen();
    let options = vec!["Save Baseline", "Compare to Baseline", "Back"];
    println!();
    ui::info("Metrics");
    for (i, opt) in options.iter().enumerate() {
        println!("  [{}] {}", i + 1, opt);
    }
    print!("\nSelection (1-3): ");
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    let choice = match s.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= 3 => n - 1,
        _ => return,
    };

    match choice {
        0 => {
            if metrics::save_baseline().is_ok() {
                ui::success("Baseline saved to baseline.json.");
            }
        }
        1 => match metrics::compare() {
            Ok(s) => ui::normal(&s),
            Err(e) => ui::error(&format!("{}", e)),
        },
        _ => {}
    }
    pause();
}

fn full_undo(backup: &backup::BackupManager) {
    clear_screen();
    ui::destructive("Restore all registry and power settings from backup?");
    let confirm = menu_confirm();
    if confirm {
        match backup.restore_all() {
            Ok(_) => ui::success("All settings restored."),
            Err(e) => ui::error(&format!("Restore failed: {}", e)),
        }
    }
    pause();
}
