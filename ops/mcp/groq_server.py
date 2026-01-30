#!/usr/bin/env python3
import sys
import json
import os
import urllib.request
import urllib.error

# --- Configuration ---
API_BASE = "https://api.groq.com/openai/v1"
# Maps tool names to specific model IDs
MODELS = {
    "ask_groq_flash": "llama3-8b-8192",
    "ask_groq_pro": "llama3-70b-8192"
}

def log(msg):
    print(f"[Groq Sidecar] {msg}", file=sys.stderr)

def get_api_key():
    # Sanitizer-bypass prefix used in .env
    key = os.getenv("GEMINI_CLI_GROQ_KEY") or os.getenv("GROQ_API_KEY")
    if not key:
        raise ValueError("Missing Groq API Key. Set GEMINI_CLI_GROQ_KEY or GROQ_API_KEY.")
    return key

def generate_content(model, prompt, system_instruction=None):
    api_key = get_api_key()
    url = f"{API_BASE}/chat/completions"
    
    messages = []
    if system_instruction:
        messages.append({"role": "system", "content": system_instruction})
    messages.append({"role": "user", "content": prompt})

    payload = {
        "model": model,
        "messages": messages,
        "temperature": 0.3
    }

    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=data, method="POST")
    req.add_header("Content-Type", "application/json")
    req.add_header("Authorization", f"Bearer {api_key}")

    try:
        with urllib.request.urlopen(req) as response:
            result = json.loads(response.read().decode("utf-8"))
            try:
                content = result["choices"][0]["message"]["content"]
                return content
            except (KeyError, IndexError):
                return f"Error: Unexpected response format. Raw: {json.dumps(result)}"
    except urllib.error.HTTPError as e:
        error_body = e.read().decode('utf-8')
        return f"Groq API Error {e.code}: {error_body}"
    except Exception as e:
        return f"System Error: {str(e)}"

def list_tools():
    return [
        {
            "name": "ask_groq_flash",
            "description": "Ask Groq (Llama 3 8B) a question. Extremely fast (500+ tokens/sec). Use for real-time validation and simple queries.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt": {"type": "string", "description": "The prompt to send to Groq."},
                    "system_instruction": {"type": "string", "description": "Optional system instruction."}
                },
                "required": ["prompt"]
            }
        },
        {
            "name": "ask_groq_pro",
            "description": "Ask Groq (Llama 3 70B) a question. High intelligence, high speed. Use for code generation and logic analysis.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt": {"type": "string", "description": "The prompt to send to Groq."},
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
    log("Starting Groq Sidecar...")
    while True:
        try:
            line = sys.stdin.readline()
            if not line:
                break
            
            request = json.loads(line)
            
            if request.get("method") == "tools/list":
                result = {"tools": list_tools()}
                response = {"jsonrpc": "2.0", "id": request.get("id"), "result": result}
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
                        "result": {"content": content, "isError": False}
                    }
                except Exception as e:
                    response = {
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "error": {"code": -32603, "message": str(e)}
                    }
                print(json.dumps(response), flush=True)
                
            elif request.get("method") == "initialize":
                 response = {
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {"tools": {}},
                        "serverInfo": {"name": "groq-sidecar", "version": "1.0.0"}
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
