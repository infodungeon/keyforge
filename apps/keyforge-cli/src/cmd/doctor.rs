// apps/keyforge-cli/src/cmd/doctor.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use clap::Args;
use std::path::Path;
use sysinfo::System;
use std::time::Duration;
use crate::constants::DEFAULT_HIVE_URL;

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {}

// [Fixed] Made async to avoid blocking reqwest
pub async fn run(_args: DoctorArgs, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🩺 KeyForge Doctor");
    eprintln!("========================================");

    // 0. Build Info
    let (git_hash, build_date) = keyforge_infra::get_build_info();
    eprintln!("🏷️  Version");
    eprintln!("   Build Hash:  {}", git_hash);
    eprintln!("   Build Date:  {}", build_date);
    eprintln!();

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

    // 1b. Toolchain Check
    eprintln!("\n🛠️  Toolchain");
    check_tool("rustc");
    check_tool("cargo");
    check_tool("node");
    check_tool("npm");
    check_tool("keyforge-agent"); // [Fixed] Check for sidecar binary

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
        ("system/weights/default_costmatrix.json", false), // Use json not mpk for local
        ("system/config/keycodes.json", false),
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

    // 5. Hive API Connectivity
    eprintln!("\n🐝 Hive API");
    let hive_url = std::env::var("KEYFORGE_HIVE_URL").unwrap_or_else(|_| DEFAULT_HIVE_URL.to_string());
    
    // [Fixed] Async Client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    
    match client.get(format!("{}/health", hive_url)).send().await {
        Ok(res) => {
            if res.status().is_success() {
                eprintln!("   ✅ Reachability: OK ({})", hive_url);
            } else {
                eprintln!("   ❌ Reachability: FAILED (Status: {})", res.status());
                all_good = false;
            }
        }
        Err(e) => {
            eprintln!("   ⚠️  Reachability: FAILED ({}) - Is Hive running?", e);
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

fn check_tool(name: &str) {
    match std::process::Command::new(name)
        .arg("--version")
        .output()
    {
        Ok(out) => {
            let ver = String::from_utf8_lossy(&out.stdout)
                .trim()
                .split_whitespace()
                .nth(1)
                .unwrap_or("?")
                .to_string();
            eprintln!("   ✅ {:<15} {}", name, ver);
        }
        Err(_) => {
            eprintln!("   ❌ {:<15} Not Found (Sidecar Required)", name);
        }
    }
}
