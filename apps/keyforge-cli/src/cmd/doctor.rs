#![allow(clippy::print_stdout, clippy::print_stderr)]
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

use crate::constants::DEFAULT_HIVE_URL;
use clap::Args;
use std::time::Duration;
use sysinfo::System;

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {}

use keyforge_boundary::SafePath;

// [Fixed] Made async to avoid blocking reqwest
pub async fn run(_args: DoctorArgs, root: &SafePath) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🩺 KeyForge Doctor");
    eprintln!("========================================");

    print_build_info();
    print_system_info();
    print_toolchain_info();
    print_cpu_info();

    let workspace_ok = check_workspace_integrity(root);
    let hive_ok = check_hive_connectivity().await;

    eprintln!("\n========================================");
    if workspace_ok && hive_ok {
        eprintln!("✅ All systems operational.");
    } else {
        eprintln!("⚠️  Issues detected. Please check the output above.");
    }

    Ok(())
}

fn print_build_info() {
    let (git_hash, build_date) = keyforge_infra::get_build_info();
    eprintln!("🏷️  Version");
    eprintln!("   Build Hash:  {git_hash}");
    eprintln!("   Build Date:  {build_date}");
    eprintln!();
}

fn print_system_info() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os = System::name().unwrap_or("Unknown".into());
    let os_ver = System::os_version().unwrap_or("?".into());
    let mem_total = sys.total_memory() / 1024 / 1024;
    let mem_used = sys.used_memory() / 1024 / 1024;

    eprintln!("🖥️  System");
    eprintln!("   OS:       {os} {os_ver}");
    eprintln!("   Memory:   {mem_used} / {mem_total} MB");
}

fn print_toolchain_info() {
    eprintln!("\n🛠️  Toolchain");
    check_tool("rustc");
    check_tool("cargo");
    check_tool("node");
    check_tool("npm");
    check_tool("keyforge-agent"); // [Fixed] Check for sidecar binary
}

fn print_cpu_info() {
    eprintln!("\n⚡ Processor");
    let cpu_count = num_cpus::get();
    eprintln!("   Cores:    {cpu_count}");

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
}

fn check_workspace_integrity(root: &SafePath) -> bool {
    eprintln!("\n📂 Workspace");
    eprintln!("   Root:     {root}");

    let required = [
        ("system/keyboards", true),
        ("system/corpora", true),
        ("system/weights", true),
        ("system/weights/default_costmatrix.json", false), // Use json not mpk for local
        ("system/config/keycodes.json", false),
        ("agent.key.age", false),
    ];

    let mut all_good = true;
    for (item, is_dir) in required {
        let p = root.as_path().join(item);
        if p.exists() {
            let matches_type = if is_dir { p.is_dir() } else { p.is_file() };
            if matches_type {
                eprintln!("   ✅ Found: {item}");
            } else {
                eprintln!(
                    "   ❌ Wrong type (target is {}): {}",
                    if is_dir { "dir" } else { "file" },
                    item
                );
                all_good = false;
            }
        } else if !std::path::Path::new(item)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("age"))
        {
            eprintln!("   ❌ Missing: {item}");
            all_good = false;
        } else {
            eprintln!("   ℹ️ Optional: {item} (used for signed remote jobs)");
        }
    }
    all_good
}

async fn check_hive_connectivity() -> bool {
    eprintln!("\n📡 Network");
    let hive_url =
        std::env::var("KEYFORGE_HIVE_URL").unwrap_or_else(|_| DEFAULT_HIVE_URL.to_string());
    eprintln!("   Hive:     {hive_url}");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build();

    if let Ok(c) = client {
        match c.get(format!("{hive_url}/health")).send().await {
            Ok(resp) if resp.status().is_success() => {
                eprintln!("   ✅ Connection: OK");
                true
            }
            Ok(resp) => {
                eprintln!("   ❌ Connection: Failed (HTTP {})", resp.status());
                false
            }
            Err(e) => {
                eprintln!("   ❌ Connection: Error ({e})");
                false
            }
        }
    } else {
        eprintln!("   ❌ HTTP Client Error");
        false
    }
}

fn check_tool(name: &str) {
    let output = std::process::Command::new(name).arg("--version").output();

    match output {
        Ok(out) => {
            let ver = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .unwrap_or("?")
                .to_string();
            eprintln!("   ✅ {name:<12} {ver}");
        }
        Err(_) => {
            eprintln!("   ❌ {name:<12} Not found in PATH");
        }
    }
}
