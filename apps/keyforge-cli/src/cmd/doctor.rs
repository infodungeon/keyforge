use clap::Args;
use std::path::Path;
use sysinfo::System;

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {}

pub fn run(_args: DoctorArgs, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🩺 KeyForge Doctor");
    eprintln!("========================================");

    // 1. System Check
    let mut sys = System::new_all();
    sys.refresh_all();

    let os = System::name().unwrap_or("Unknown".into());
    let os_ver = System::os_version().unwrap_or("?".into());
    let mem_total = sys.total_memory() / 1024 / 1024;
    let mem_used = sys.used_memory() / 1024 / 1024;

    eprintln!("🖥️  System");
    eprintln!("   OS:       {} {}", os, os_ver);
    eprintln!("   Memory:   {} / {} MB", mem_used, mem_total);

    // 2. CPU Capabilities
    eprintln!("\n⚡ Processor");
    let cpu_count = num_cpus::get();
    eprintln!("   Cores:    {}", cpu_count);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            eprintln!("   AVX2:     ✅ Supported (High Performance Mode)");
        } else {
            eprintln!("   AVX2:     ❌ Not Detected (Scalar Fallback Mode)");
        }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        eprintln!("   Arch:     Non-x86 (Standard Mode)");
    }

    // 3. Workspace Integrity
    eprintln!("\n📂 Workspace");
    eprintln!("   Root:     {:?}", root);

    let required = [
        ("system/keyboards", true),
        ("system/corpora", true),
        ("system/weights", true),
        ("system/weights/default_costmatrix.mpk.zst", false),
        ("system/config/keycodes.mpk.zst", false),
    ];

    let mut all_good = true;
    for (item, is_dir) in required {
        let p = root.join(item);
        if p.exists() {
            let matches_type = if is_dir { p.is_dir() } else { p.is_file() };
            if matches_type {
                eprintln!("   ✅ Found: {}", item);
            } else {
                eprintln!(
                    "   ❌ Wrong type (target is {}): {}",
                    if is_dir { "dir" } else { "file" },
                    item
                );
                all_good = false;
            }
        } else {
            eprintln!("   ❌ Missing: {}", item);
            all_good = false;
        }
    }

    // 4. Write Permissions
    let test_file = root.join(".write_test");
    if test_file.exists() {
        eprintln!("   ⚠️  Warning: .write_test already exists, skipping write check.");
    } else {
        match std::fs::write(&test_file, "test") {
            Ok(_) => {
                eprintln!("   ✅ Write Access: OK");
                let _ = std::fs::remove_file(test_file);
            }
            Err(e) => {
                eprintln!("   ❌ Write Access: FAILED ({})", e);
                all_good = false;
            }
        }
    }

    eprintln!("\n========================================");
    if all_good {
        eprintln!("✨ System Healthy. Ready to Forge.");
        Ok(())
    } else {
        Err("Issues detected. Run 'keyforge init' to repair workspace.".into())
    }
}
