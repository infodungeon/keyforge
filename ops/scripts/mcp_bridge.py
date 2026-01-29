#!/usr/bin/env python3
import sys
import os
import shutil
import time

# Load .env if it exists
def load_env():
    env_path = "/home/robert/Documents/KeyboardLayouts/DataDrivenAnalysis/keyforge/.env"
    if os.path.exists(env_path):
        with open(env_path, "r") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#"):
                    key, _, val = line.partition("=")
                    # Strip quotes and whitespace
                    os.environ[key.strip()] = val.strip().strip('"').strip("'")

load_env()

DEBUG_LOG = "/tmp/mcp_debug.log"
def log(msg):
    try:
        with open(DEBUG_LOG, "a") as f:
            f.write(f"[{time.ctime()}] [PID {os.getpid()}] {msg}\n")
    except:
        pass

def main():
    if len(sys.argv) < 2:
        print("Usage: mcp_bridge.py <target> [args...]", file=sys.stderr)
        sys.exit(1)

    target = sys.argv[1]
    
    # Bypass Gemini CLI masking by preferring KF_GITHUB_TOKEN
    clean_token = os.getenv("KF_GITHUB_TOKEN") or os.getenv("GITHUB_PERSONAL_ACCESS_TOKEN")
    
    log(f"Bridge requested for: {' '.join(sys.argv[1:])}")
    log(f"Clean token source: {'KF_GITHUB_TOKEN' if os.getenv('KF_GITHUB_TOKEN') else 'GITHUB_PERSONAL_ACCESS_TOKEN'}")
    log(f"Token present: {'Yes' if clean_token else 'No'}")

    env = os.environ.copy()
    if clean_token:
        env["GITHUB_PERSONAL_ACCESS_TOKEN"] = clean_token
        env["GITHUB_TOKEN"] = clean_token

    if target == "github":
        node_bin = shutil.which("node") or "/usr/bin/node"
        github_bin = "/home/robert/.npm-global/lib/node_modules/@modelcontextprotocol/server-github/dist/index.js"
        if not os.path.exists(github_bin):
            github_bin = os.path.join(os.getcwd(), "node_modules/@modelcontextprotocol/server-github/dist/index.js")
            
        if not os.path.exists(github_bin):
            log(f"Error: GitHub MCP server not found at {github_bin}")
            sys.exit(1)
            
        cmd = [node_bin, github_bin]
    else:
        resolved_bin = shutil.which(target)
        if not resolved_bin:
            if os.path.isabs(target) and os.path.exists(target):
                resolved_bin = target
            else:
                log(f"Error: Command {target} not found")
                sys.exit(1)
        cmd = [resolved_bin] + sys.argv[2:]

    log(f"Executing replacement (execvpe): {' '.join(cmd)}")
    
    # Replace the current process with the target command
    # This is the most robust way to bridge stdio servers
    os.execvpe(cmd[0], cmd, env)

if __name__ == "__main__":
    main()
