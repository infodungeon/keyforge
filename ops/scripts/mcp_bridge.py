#!/usr/bin/env python3
import sys
import subprocess
import os
import signal
import psutil
import time

def find_and_kill_existing(target_name):
    """Finds and kills existing processes matching the target name to ensure a clean start."""
    for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
        try:
            cmdline = proc.info.get('cmdline')
            if not cmdline: continue
            
            # Check if this process is the target binary
            # We look for the binary name at the end of the path
            executable = cmdline[0]
            if target_name in os.path.basename(executable):
                # Don't kill ourselves if we somehow match (unlikely as we are python)
                if proc.pid == os.getpid():
                    continue
                    
                # Kill it
                # print(f"[*] Bridge: Killing existing {target_name} (PID {proc.pid})...", file=sys.stderr)
                try:
                    os.kill(proc.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                except Exception:
                    # Force kill if needed
                    try:
                        os.kill(proc.pid, signal.SIGKILL)
                    except:
                        pass
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue
    
    # Brief pause to allow OS to clean up
    time.sleep(0.1)

def main():
    if len(sys.argv) < 2:
        print("Usage: mcp_bridge.py <command> [args...]", file=sys.stderr)
        sys.exit(1)

    target_cmd = sys.argv[1]
    target_args = sys.argv[2:]
    
    target_name = os.path.basename(target_cmd)

    # 1. Ensure single instance (The "Connection Closed" fix)
    # We enforce that only THIS session's instance runs.
    find_and_kill_existing(target_name)

    # 2. Start the server
    # We pass stdin/stdout through to the parent (Gemini)
    # This removes the "Fake Handshake" issue - Gemini talks directly to the tool.
    try:
        proc = subprocess.Popen(
            [target_cmd] + target_args,
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr, # Pass stderr through for logs
            bufsize=0 # Unbuffered
        )
        
        # Wait for it to finish (driven by Gemini closing stdin)
        proc.wait()
        
    except Exception as e:
        print(f"Bridge Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
