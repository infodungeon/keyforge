#!/usr/bin/env python3
import sys
import subprocess
import os
import signal
import psutil
import time
import shutil
import json
import threading
import queue
import requests

# Load .env if it exists
def load_env():
    # Use absolute path to ensure it's found
    env_path = "/home/robert/Documents/KeyboardLayouts/DataDrivenAnalysis/keyforge/.env"
    if os.path.exists(env_path):
        with open(env_path, "r") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#"):
                    key, _, val = line.partition("=")
                    os.environ[key.strip()] = val.strip().strip('"').strip("'")

load_env()

DEBUG_LOG = "/tmp/mcp_debug.log"
# Look for KF_GITHUB_TOKEN first to bypass CLI masking
GITHUB_TOKEN_RAW = os.getenv("KF_GITHUB_TOKEN") or os.getenv("GITHUB_PERSONAL_ACCESS_TOKEN")
GITHUB_TOKEN = GITHUB_TOKEN_RAW.strip('"').strip("'") if GITHUB_TOKEN_RAW else None

def log(msg):
    try:
        with open(DEBUG_LOG, "a") as f:
            f.write(f"[{time.ctime()}] [PID {os.getpid()}] {msg}\n")
    except:
        pass

log(f"Token loaded: {'None' if not GITHUB_TOKEN else len(GITHUB_TOKEN)}")

def find_and_kill_existing(target_id):
    """Kills any previous bridge instance for this specific target id."""
    my_pid = os.getpid()
    for proc in psutil.process_iter(['pid', 'cmdline']):
        try:
            pid = proc.info['pid']
            if pid == my_pid: continue
            cmdline = proc.info.get('cmdline')
            if not cmdline: continue
            
            # Check if this is another bridge for the SAME target
            if "mcp_bridge.py" in " ".join(cmdline) and target_id in " ".join(cmdline):
                try:
                    os.kill(pid, signal.SIGTERM)
                except:
                    pass
        except:
            continue
    time.sleep(0.1)

def handle_stdin_to_queue(iq):
    for line in sys.stdin:
        try:
            iq.put(json.loads(line))
        except:
            continue

def run_copilot_bridge():
    log("Starting Copilot Remote Mode")
    input_queue = queue.Queue()
    threading.Thread(target=handle_stdin_to_queue, args=(input_queue,), daemon=True).start()
    
    COPILOT_URL = "https://api.github.com/copilot/chat/completions" 
    
    session = requests.Session()
    session.headers.update({
        "Authorization": f"Bearer {GITHUB_TOKEN}",
        "Content-Type": "application/json"
    })
    
    while True:
        try:
            msg = input_queue.get(timeout=1)
            response = session.post(COPILOT_URL, json=msg)
            if response.status_code == 200:
                sys.stdout.write(json.dumps(response.json()) + "\n")
                sys.stdout.flush()
            else:
                error_resp = {"jsonrpc": "2.0", "id": msg.get("id"), "error": {"code": -32000, "message": f"Remote Error: {response.status_code}"}}
                sys.stdout.write(json.dumps(error_resp) + "\n")
                sys.stdout.flush()
        except queue.Empty:
            continue
        except Exception as e:
            log(f"Copilot Error: {e}")

def main():
    if len(sys.argv) < 2:
        print("Usage: mcp_bridge.py <target> [args...]", file=sys.stderr)
        sys.exit(1)

    target = sys.argv[1]
    
    # Use the target + args as a unique ID for cleanup
    target_id = " ".join(sys.argv[1:])
    find_and_kill_existing(target_id)
    
    log(f"Bridge requested for: {target_id}")

    if target == "copilot":
        run_copilot_bridge()
        return

    # Handle standard binaries and github node server
    env = os.environ.copy()
    cmd = []
    
    if target == "github":
        log("Starting GitHub Mode")
        if GITHUB_TOKEN:
            # Map the clean variable to the one expected by the server
            env["GITHUB_PERSONAL_ACCESS_TOKEN"] = GITHUB_TOKEN
            env["GITHUB_TOKEN"] = GITHUB_TOKEN
        
        node_bin = shutil.which("node") or "/usr/bin/node"
        github_bin = "/home/robert/.npm-global/lib/node_modules/@modelcontextprotocol/server-github/dist/index.js"
        if not os.path.exists(github_bin):
            # Fallback to local node_modules if not global
            github_bin = os.path.join(os.getcwd(), "node_modules/@modelcontextprotocol/server-github/dist/index.js")
            
        if not os.path.exists(github_bin):
            log(f"Error: GitHub MCP server not found at {github_bin}")
            sys.exit(1)
            
        cmd = [node_bin, github_bin]
    else:
        resolved_bin = shutil.which(target)
        if not resolved_bin:
            # If target is already an absolute path that exists, use it
            if os.path.isabs(target) and os.path.exists(target):
                resolved_bin = target
            else:
                log(f"Error: Command {target} not found")
                sys.exit(1)
        cmd = [resolved_bin] + sys.argv[2:]

    log(f"Executing: {' '.join(cmd)}")

    try:
        proc = subprocess.Popen(
            cmd,
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            env=env,
            bufsize=0,
            universal_newlines=True
        )
        
        def sig_handler(sig, frame):
            proc.send_signal(sig)
            sys.exit(0)
            
        signal.signal(signal.SIGTERM, sig_handler)
        signal.signal(signal.SIGINT, sig_handler)
        
        status = proc.wait()
        log(f"Target {target} exited with status {status}")
        sys.exit(status)
    except Exception as e:
        log(f"Bridge Execution Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()