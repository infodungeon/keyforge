#!/usr/bin/env python3
import sys
import json
import os
import urllib.request
import urllib.error

# --- Configuration ---
API_BASE = "https://generativelanguage.googleapis.com/v1beta"
# Maps tool names to specific model IDs
MODELS = {
    "ask_studio_flash": "models/gemini-3-flash-preview"
}

def log(msg):
    print(f"[Studio Sidecar] {msg}", file=sys.stderr)

def get_api_key():
    # Supports the Rename Bridge pattern
    key = os.getenv("GEMINI_API_KEY") or os.getenv("KF_STUDIO_KEY")
    if not key:
        raise ValueError("Missing API Key. Set GEMINI_API_KEY or KF_STUDIO_KEY.")
    return key

def generate_content(model, prompt, system_instruction=None):
    api_key = get_api_key()
    url = f"{API_BASE}/{model}:generateContent?key={api_key}"
    
    payload = {
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    }
    
    if system_instruction:
        payload["systemInstruction"] = {
            "parts": [{"text": system_instruction}]
        }

    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=data, method="POST")
    req.add_header("Content-Type", "application/json")

    try:
        with urllib.request.urlopen(req) as response:
            result = json.loads(response.read().decode("utf-8"))
            # Extract text from the first candidate
            try:
                text = result["candidates"][0]["content"]["parts"][0]["text"]
                model_version = result.get("modelVersion", "unknown")
                return f"[API Reported Version: {model_version}]\n\n{text}"
            except (KeyError, IndexError):
                return f"Error: No content returned. Raw response: {json.dumps(result)}"
    except urllib.error.HTTPError as e:
        return f"API Error {e.code}: {e.read().decode('utf-8')}"
    except Exception as e:
        return f"System Error: {str(e)}"

def list_tools():
    return [
        {
            "name": "ask_studio_flash",
            "description": "Ask Gemini 3.0 Flash (Preview) a question via AI Studio. High speed, high throughput. Use for offloading summaries, data extraction, and iterative background tasks.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt": {"type": "string", "description": "The prompt to send to the model."},
                    "system_instruction": {"type": "string", "description": "Optional system instruction."}
                },
                "required": ["prompt"]
            }
        }
    ]

def handle_call_tool(name, arguments):
    if name in MODELS:
        model_id = MODELS[name]
        prompt = arguments.get("prompt")
        system_instruction = arguments.get("system_instruction")
        
        log(f"Calling {model_id}...")
        response_text = generate_content(model_id, prompt, system_instruction)
        
        return [
            {
                "type": "text",
                "text": response_text
            }
        ]
    else:
        raise ValueError(f"Unknown tool: {name}")

def main():
    # Basic MCP Stdio Loop
    log("Starting Studio Sidecar (Flash-Only Mode)...")
    while True:
        try:
            line = sys.stdin.readline()
            if not line:
                break
            
            request = json.loads(line)
            
            # Simple JSON-RPC 2.0 handling
            if request.get("method") == "tools/list":
                result = {"tools": list_tools()}
                response = {
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": result
                }
                print(json.dumps(response), flush=True)
                
            elif request.get("method") == "tools/call":
                params = request.get("params", {})
                name = params.get("name")
                args = params.get("arguments", {})
                
                try:
                    content = handle_call_tool(name, args)
                    response = {
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "result": {
                            "content": content,
                            "isError": False
                        }
                    }
                except Exception as e:
                    response = {
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "error": {
                            "code": -32603,
                            "message": str(e)
                        }
                    }
                
                print(json.dumps(response), flush=True)
                
            elif request.get("method") == "initialize":
                 response = {
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "studio-sidecar",
                            "version": "1.1.0"
                        }
                    }
                }
                 print(json.dumps(response), flush=True)
            
            elif request.get("method") == "notifications/initialized":
                pass
                
            else:
                pass
                
        except json.JSONDecodeError:
            continue
        except Exception as e:
            log(f"Fatal Loop Error: {e}")
            break

if __name__ == "__main__":
    main()
