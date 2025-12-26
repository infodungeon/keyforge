#!/usr/bin/env python3
import json
import subprocess
import shutil
import os
import sys
import toml

def isolate_project(app_name, output_dir):
    # 1. Get Workspace Metadata
    print(f"🔍 Analyzing dependencies for {app_name}...")
    meta_json = subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"], 
        text=True
    )
    meta = json.loads(meta_json)
    workspace_root = meta["workspace_root"]
    
    # 2. Find Target Package and Local Dependencies
    full_meta_json = subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1"], 
        text=True
    )
    full_meta = json.loads(full_meta_json)
    
    packages = {p["id"]: p for p in full_meta["packages"]}
    target_pkg = next((p for p in full_meta["packages"] if p["name"] == app_name), None)
    
    if not target_pkg:
        print(f"❌ Package '{app_name}' not found in workspace.")
        sys.exit(1)

    local_members = set()
    
    def collect_local_deps(node_id):
        pkg = packages[node_id]
        if any(m == pkg["id"] for m in meta["workspace_members"]):
            local_members.add(pkg["manifest_path"])
            node = next(n for n in full_meta["resolve"]["nodes"] if n["id"] == node_id)
            for dep in node["dependencies"]:
                collect_local_deps(dep)

    collect_local_deps(target_pkg["id"])

    # 3. Prepare Output Directory
    if os.path.exists(output_dir):
        shutil.rmtree(output_dir)
    os.makedirs(output_dir)

    print(f"📦 Staging {len(local_members)} local members to {output_dir}...")

    # 4. Copy Member Directories
    final_members_paths = []
    
    for manifest_path in local_members:
        rel_manifest = os.path.relpath(manifest_path, workspace_root)
        member_dir = os.path.dirname(rel_manifest)
        
        src_dir = os.path.join(workspace_root, member_dir)
        dst_dir = os.path.join(output_dir, member_dir)
        
        os.makedirs(os.path.dirname(dst_dir), exist_ok=True)
        shutil.copytree(src_dir, dst_dir, ignore=shutil.ignore_patterns('target', '.git', 'node_modules'))
        final_members_paths.append(member_dir)

    # 5. Generate Synthesized Workspace Cargo.toml
    root_toml_path = os.path.join(workspace_root, "Cargo.toml")
    with open(root_toml_path, "r") as f:
        root_config = toml.load(f)

    root_config["workspace"]["members"] = final_members_paths
    if "default-members" in root_config["workspace"]:
        del root_config["workspace"]["default-members"]

    with open(os.path.join(output_dir, "Cargo.toml"), "w") as f:
        toml.dump(root_config, f)

    # 6. Copy Lockfile
    shutil.copy(os.path.join(workspace_root, "Cargo.lock"), os.path.join(output_dir, "Cargo.lock"))
    
    print(f"✅ Isolation complete. Context ready at {output_dir}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python isolate.py <crate_name> <output_dir>")
        sys.exit(1)
    isolate_project(sys.argv[1], sys.argv[2])
