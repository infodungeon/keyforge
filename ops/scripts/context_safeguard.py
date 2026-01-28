#!/usr/bin/env python3
import sys
import json
import re

def main():
    try:
        input_data = json.load(sys.stdin)
    except Exception:
        # If we can't parse input, fail safe and allow (or block depending on policy)
        print(json.dumps({"continue": True}))
        return

    tool_name = input_data.get("tool_name")
    tool_input = input_data.get("tool_input", {})

    # Rule 1: Block search_file_content on root
    if tool_name == "search_file_content":
        dir_path = tool_input.get("dir_path", ".")
        if dir_path in [".", "./", "/"]:
            print(json.dumps({
                "continue": False,
                "decision": "block",
                "reason": "CRITICAL: Searching the root directory ('.') is forbidden to prevent context overflow. Please target a subdirectory (e.g., 'libs/', 'apps/')."
            }))
            return

    # Rule 2: Block grep on root in run_shell_command
    if tool_name == "run_shell_command":
        command = tool_input.get("command", "")
        # Match grep patterns that target root
        # e.g., "grep -r pattern .", "grep pattern ./", etc.
        if re.search(r"\bgrep\b.*?\s\.(?:\s|$|/)", command):
            print(json.dumps({
                "continue": False,
                "decision": "block",
                "reason": "CRITICAL: Running 'grep' on the root directory ('.') is forbidden. Use 'search_file_content' on a specific subdirectory or a more targeted grep command."
            }))
            return

    # Rule 3: Block ripgrep (rg) on root
    if tool_name == "run_shell_command":
        command = tool_input.get("command", "")
        if re.search(r"\brg\b.*?\s\.(?:\s|$|/)", command):
            print(json.dumps({
                "continue": False,
                "decision": "block",
                "reason": "CRITICAL: Running 'rg' on the root directory ('.') is forbidden. Target a subdirectory."
            }))
            return

    # Default: Allow
    print(json.dumps({"continue": True}))

if __name__ == "__main__":
    main()
