#!/usr/bin/env python3
import os
import subprocess
import yaml
import sys

def check_process(name):
    try:
        output = subprocess.check_output(["ps", "aux"]).decode()
        return name in output
    except:
        return False

def main():
    config_path = "ops/config/mcp_gateway_config.yaml"
    if not os.path.exists(config_path):
        print(f"FAILED: {config_path} not found")
        sys.exit(1)

    with open(config_path, 'r') as f:
        config = yaml.safe_load(f)

    servers = config.get("mcpServers", {})
    all_ok = True

    print("=== MCP Health Check ===")
    for server_name in servers:
        cmd = servers[server_name].get("command")
        is_running = check_process(os.path.basename(cmd))
        status = "✅ RUNNING" if is_running else "❌ DOWN"
        print(f"{server_name:10}: {status}")
        if not is_running:
            all_ok = False

    if not all_ok:
        print("\nSuggestions:")
        print("1. Run 'just mcp-restart' (if available)")
        print("2. Check if mcp-proxy-tool is installed")
        sys.exit(1)
    else:
        print("\nAll MCP servers are active.")

if __name__ == "__main__":
    main()