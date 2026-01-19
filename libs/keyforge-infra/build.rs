// Copyright (c) 2025 KeyForge Contributors
//
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
//! Build script for `keyforge-infra` to inject build metadata and Git information.

use std::fs;
use std::path::Path;
use std::process::Command;

#[allow(clippy::unwrap_used, clippy::expect_used)]
fn main() {
    let out_dir =
        std::env::var_os("OUT_DIR").expect("OUT_DIR not set; this script must be run via cargo");
    let dest_path = Path::new(&out_dir).join("build_info.rs");

    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());

    let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let content = format!(
        "/// The short Git commit hash identifying the source code used for this build.\npub const GIT_HASH: &str = \"{git_hash}\";\n/// The date and time when this workspace was compiled.\npub const BUILD_DATE: &str = \"{build_date}\";\n"
    );

    fs::write(&dest_path, content).expect("Failed to write build_info.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=build.rs");
}
