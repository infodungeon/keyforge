#!/usr/bin/env python3
import sys
import subprocess
import os
import signal
import psutil
import time
import shutil

DEBUG_LOG = "/tmp/mcp_debug.log"

def log(msg):
    try:
        with open(DEBUG_LOG, "a") as f:
            f.write(f"[{time.ctime()}] [PID {os.getpid()}] {msg}\n")
    except:
        pass

def find_and_kill_existing(target_id):
    """Kills any previous bridge instance for this specific target id."""
    my_pid = os.getpid()
    for proc in psutil.process_iter(['pid', 'cmdline']):
        try:
            pid = proc.info['pid']
            if pid == my_pid: continue
            cmdline = proc.info.get('cmdline')
            if not cmdline: continue
            
            cmd_str = " ".join(cmdline)
            # Match the target_id to avoid killing different bridges
            if "mcp_bridge.py" in cmd_str and target_id in cmd_str:
                if pid == os.getppid(): continue
                try:
                    os.kill(pid, signal.SIGTERM)
                except:
                    pass
        except:
            continue
    time.sleep(0.1)

def main():
    if len(sys.argv) < 2:
        print("Usage: mcp_bridge.py <target_bin> [args...]", file=sys.stderr)
        sys.exit(1)

    target_bin = sys.argv[1]
    # Use the full command line as the ID for stabilization
    target_id = " ".join(sys.argv[1:])
    
    find_and_kill_existing(target_id)
    log(f"Bridge requested for: {target_id}")

    resolved_bin = shutil.which(target_bin)
    if not resolved_bin:
        log(f"Error: Command {target_bin} not found")
        sys.exit(1)

    cmd = [resolved_bin] + sys.argv[2:]

    try:
        proc = subprocess.Popen(
            cmd,
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            bufsize=0
        )
        
        def sig_handler(sig, frame):
            proc.send_signal(sig)
            sys.exit(0)
            
        signal.signal(signal.SIGTERM, sig_handler)
        signal.signal(signal.SIGINT, sig_handler)
        
        status = proc.wait()
        log(f"Target {target_bin} exited with status {status}")
        sys.exit(status)
    except Exception as e:
        log(f"Bridge Execution Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()