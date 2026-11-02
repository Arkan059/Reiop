# Reiop

**Reiop** is a standalone, high-performance Windows optimization and diagnostic CLI tool written entirely in Rust. It has zero dependencies on PowerShell or external scripts, utilizing native Windows APIs and the Registry directly.

---

## What It Does

- **Diagnostics & HTML Report:** Scans your hardware (CPU, GPU, RAM, Disk, Refresh Rate, GPU Driver Age, CPU Temps) and Windows settings. It calculates a Performance Score out of 100 and generates a beautifully styled dark-aquatic HTML report with animated swimming fish.
- **Tweak Profiles:** Apply "Balanced" or "Competitive" performance tweaks with one click, or pick specific optimizations manually (Power Plan, HAGS, Game Mode, MMCSS, Mouse Accel, etc.).
- **Startup Manager:** View and safely disable bloated startup applications.
- **Cleanup Center:** Calculate and safely delete temporary files, shader caches, and Windows Update leftovers.
- **Windows Repairs:** Run native `DISM` and `SFC` system health scans directly from the menu.
- **VBS Toggle:** Easily disable Virtualization-Based Security (Core Isolation) to optimize for gaming.
- **Metrics Comparison:** Save a system baseline and compare RAM usage, process count, and CPU load before and after tweaking.
- **Full Undo:** Automatically saves previous states to `backup.json` before applying tweaks, allowing you to revert all changes instantly if needed.

---

## Safety & Notices

- **Requires Administrator Privileges:** The tool will automatically ask for UAC elevation to modify system settings.
- **SmartScreen Warning:** Because the executable is unsigned, Windows Defender/SmartScreen may flag it. Click **"More info" -> "Run anyway"**.
- **Backups:** The tool automatically creates a `backup.json` file in its directory. Do not delete this file if you want the ability to undo your changes.

---

## Contact

Created by **arkanquixote**
- Discord & Instagram: `arkanquixote`
