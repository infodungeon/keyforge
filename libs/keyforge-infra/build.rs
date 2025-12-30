use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_info.rs");

    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    let content = format!(
        "pub const GIT_HASH: &str = \"{}\";\npub const BUILD_DATE: &str = \"{}\";\n",
        git_hash, build_date
    );

    fs::write(&dest_path, content).unwrap();
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=build.rs");
}
