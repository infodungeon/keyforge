#!/usr/bin/env python3
import sys
import subprocess
import os
import signal
import psutil
import time
import fcntl
import shutil

def find_and_kill_existing(target_name):
    """Finds and kills existing processes matching the target name to ensure a clean start."""
    # Use a lockfile to prevent race conditions between multiple bridges starting at once
    lock_path = f"/tmp/mcp_bridge_{target_name}.lock"
    try:
        lock_file = open(lock_path, "w")
        fcntl.flock(lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except (IOError, OSError):
        # Someone else is already cleaning up or starting
        return

    for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
        try:
            cmdline = proc.info.get('cmdline')
            if not cmdline: continue
            
            # Check if this process is the target binary
            # We look for the binary name at the end of the path
            executable = cmdline[0]
            if target_name == os.path.basename(executable):
                # Don't kill ourselves or our parent bridge
                if proc.pid == os.getpid() or proc.pid == os.getppid():
                    continue
                    
                try:
                    os.kill(proc.pid, signal.SIGTERM)
                    # Wait up to 1 second for it to die
                    for _ in range(10):
                        time.sleep(0.1)
                        if not psutil.pid_exists(proc.pid):
                            break
                    else:
                        # Force kill if still alive
                        os.kill(proc.pid, signal.SIGKILL)
                except (ProcessLookupError, psutil.NoSuchProcess):
                    pass
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    
    # Brief pause to allow OS to clean up sockets/files
    time.sleep(0.2)
    
    # Keep the lock until we are done (implicitly released on exit)

def main():
    if len(sys.argv) < 2:
        print("Usage: mcp_bridge.py <command> [args...]", file=sys.stderr)
        sys.exit(1)

    target_cmd = sys.argv[1]
    
    # Handle 'status' or metadata commands if needed
    if target_cmd == "status":
        print("MCP Bridge is active.")
        sys.exit(0)

    target_args = sys.argv[2:]
    
    # Resolve the full path of the command if it's not absolute
    resolved_cmd = shutil.which(target_cmd)
    if not resolved_cmd:
        print(f"Bridge Error: Command '{target_cmd}' not found in PATH.", file=sys.stderr)
        sys.exit(1)
        
    target_name = os.path.basename(resolved_cmd)

    # 1. Ensure single instance (The "Connection Closed" fix)
    find_and_kill_existing(target_name)

    # 2. Start the server
    try:
        # We pass FDs directly. bufsize=0 is passed through to the OS level.
        # This ensures minimal latency and transparency for the MCP protocol.
        proc = subprocess.Popen(
            [resolved_cmd] + target_args,
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            bufsize=0
        )
        
        # Handle termination signals by passing them to the child
        def signal_handler(sig, frame):
            proc.send_signal(sig)
            sys.exit(0)
            
        signal.signal(signal.SIGTERM, signal_handler)
        signal.signal(signal.SIGINT, signal_handler)
        
        # Wait for it to finish (driven by Gemini closing stdin)
        status = proc.wait()
        sys.exit(status)
        
    except Exception as e:
        print(f"Bridge Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()