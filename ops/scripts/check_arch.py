#!/usr/bin/env python3
import json
import subprocess
import sys
import os

# Definition of the Layered Architecture

# Layer 0: Contract / Types
TIER_0_CONTRACT = ["keyforge-model"]

# Layer 1: Ports / Adapters
TIER_1_PORTS = ["keyforge-protocol", "keyforge-adapter"]

# Layer 2: Domain Logic (Nucleus)
TIER_2_DOMAIN = ["keyforge-physics"]

# Layer 3: Advanced Domain
TIER_3_EVOLUTION = ["keyforge-evolution"]

# Layer 4: Infrastructure
# Ideally should not depend on Compute/Physics, but current legacy does.
TIER_4_INFRA = ["keyforge-infra", "keyforge-persistence", "keyforge-security"]

# Layer 5: Orchestration
TIER_5_RUNTIME = ["keyforge-compute"]

# Apps
APPS = [
    "keyforge-agent", "keyforge-assetmgr", "keyforge-assets", 
    "keyforge-cli", "keyforge-hive", "keyforge-tui", 
    "keyforge-ui", "keyforge-wasm", "keyforge-repros", "keyforge-system-tests"
]

# Legacy Violations to be worked off (Technical Debt)
# Format: "crate_name": ["allowed_bad_dependency"]
LEGACY_EXCEPTIONS = {
    "keyforge-infra": ["keyforge-compute"], # DEBT: Infra shouldn't depend on Runtime
    "keyforge-persistence": ["keyforge-compute"], # DEBT: Persistence shouldn't depend on Runtime
    "keyforge-evolution": ["keyforge-protocol"], # DEBT: Evolution should depend on Model only? Or Protocol is okay?
}

def get_banned_list(crate_name):
    banned = []
    
    if crate_name in TIER_0_CONTRACT:
        banned = ["keyforge-protocol", "keyforge-adapter", "keyforge-physics", 
                  "keyforge-evolution", "keyforge-infra", "keyforge-persistence", 
                  "keyforge-security", "keyforge-compute", "keyforge-testing"]
                  
    elif crate_name in TIER_1_PORTS:
        banned = ["keyforge-physics", "keyforge-evolution", "keyforge-compute",
                  "keyforge-infra", "keyforge-persistence"]
                  
    elif crate_name in TIER_2_DOMAIN:
        banned = ["keyforge-evolution", "keyforge-compute", 
                  "keyforge-infra", "keyforge-persistence", "keyforge-security",
                  "keyforge-protocol", "keyforge-adapter"]
                  
    elif crate_name in TIER_3_EVOLUTION:
        banned = ["keyforge-compute", 
                  "keyforge-infra", "keyforge-persistence", "keyforge-security",
                  "keyforge-adapter"] # Protocol allowed by legacy exception for now
                  
    elif crate_name in TIER_4_INFRA:
        banned = ["keyforge-physics", "keyforge-evolution", "keyforge-compute"]
        
    return banned

def get_cargo_metadata():
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--format-version", "1"],
            capture_output=True,
            text=True,
            check=True
        )
        return json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Error running cargo metadata: {e.stderr}")
        sys.exit(1)

def check_architecture(metadata):
    print("Verifying Architectural Layer Boundaries...")
    violations = []
    
    workspace_members = metadata["workspace_members"]
    packages = {p["id"]: p for p in metadata["packages"]}
    
    for member_id in workspace_members:
        pkg = packages[member_id]
        name = pkg["name"]
        
        if name in APPS:
            continue
            
        banned = get_banned_list(name)
        allowed_legacy = LEGACY_EXCEPTIONS.get(name, [])
        
        # In `cargo metadata`, dependencies are listed with "kind".
        # We only care about kind=null (normal) or kind="build". 
        # kind="dev" should be ignored.
        
        for dep in pkg["dependencies"]:
            dep_name = dep["name"]
            dep_kind = dep.get("kind", None)
            
            # Skip dev dependencies
            if dep_kind == "dev":
                continue
            
            # We only care about workspace dependencies
            if not any(p["name"] == dep_name for p in metadata["packages"]):
                continue
                
            if dep_name in banned:
                if dep_name in allowed_legacy:
                    print(f"WARNING: Allowed Legacy Violation: '{name}' -> '{dep_name}'")
                else:
                    violations.append(f"VIOLATION: '{name}' depends on '{dep_name}' (Banned: {banned})")

    if violations:
        print("\nArchitectural Violations Found:")
        for v in violations:
            print(f"  - {v}")
        return False
        
    print("Architectural layers intact (with known exceptions).")
    return True

def main():
    metadata = get_cargo_metadata()
    if check_architecture(metadata):
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()
