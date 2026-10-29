//! Angkor Creative Suite interop: seamless switching between AngkorPix (Photoshop) and AngkorVector (Illustrator).

use std::path::{Path, PathBuf};
use std::process::Command;

/// Attempts to find the AngkorVector executable path.
pub fn find_angkorvector_exe() -> Option<PathBuf> {
    // 1. Check beside current executable
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let beside = dir.join("angkorvector.exe");
            if beside.is_file() {
                return Some(beside);
            }
            let beside_legacy = dir.join("vectorcraft.exe");
            if beside_legacy.is_file() {
                return Some(beside_legacy);
            }
        }
    }

    // 2. Check known build paths
    let candidates = [
        r"E:\clone ps\vectorcraft\target\debug\angkorvector.exe",
        r"E:\clone ps\vectorcraft\target\release\angkorvector.exe",
        r"..\vectorcraft\target\debug\angkorvector.exe",
        r"..\vectorcraft\target\release\angkorvector.exe",
    ];

    for c in &candidates {
        let p = Path::new(c);
        if p.is_file() {
            return Some(p.to_path_buf());
        }
    }

    None
}

/// Launches or brings AngkorVector to the foreground.
pub fn launch_or_focus_angkorvector() {
    if let Some(exe) = find_angkorvector_exe() {
        let _ = Command::new(&exe).spawn();
    } else {
        let vectorcraft_dir = Path::new(r"E:\clone ps\vectorcraft");
        if vectorcraft_dir.is_dir() {
            let _ = Command::new("cargo")
                .args(["run", "-p", "angkorvector"])
                .current_dir(vectorcraft_dir)
                .spawn();
        }
    }
}

