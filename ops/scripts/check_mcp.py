#!/usr/bin/env python3
import json
import subprocess
import os
import sys

CONFIG_PATH = "mcp_config.json"
BRIDGE_PATH = "ops/scripts/mcp_bridge.py"

def check_mcp():
    if not os.path.exists(CONFIG_PATH):
        print(f"FAILED: {CONFIG_PATH} not found")
        return False

    with open(CONFIG_PATH, "r") as f:
        config = json.load(f)

    all_pass = True
    servers = config.get("mcpServers", {})
    
    for name, srv in servers.items():
        args = srv.get("args", [])
        if name in ["github", "copilot"]:
            if BRIDGE_PATH not in args[0]:
                 print(f"FAILED: {name} does not use unified bridge")
                 all_pass = False
            else:
                 print(f"PASSED: {name} uses unified bridge")
        
        # Test if bridge script exists
        if not os.path.exists(BRIDGE_PATH):
            print(f"FAILED: Bridge script {BRIDGE_PATH} missing")
            return False

    return all_pass

if __name__ == "__main__":
    if check_mcp():
        print("\nMCP Configuration Stabilized Successfully.")
        sys.exit(0)
    else:
        print("\nMCP Configuration Issues Detected.")
        sys.exit(1)
