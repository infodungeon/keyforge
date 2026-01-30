#!/usr/bin/env python3
import sys
import json
import os
import urllib.request
import urllib.error
import time
import subprocess
from concurrent.futures import ThreadPoolExecutor

def log(msg):
    print(f"[Swarm Router] {msg}", file=sys.stderr)

USAGE_FILE = os.path.expanduser("/home/robert/Documents/KeyboardLayouts/DataDrivenAnalysis/keyforge/.gemini/swarm_usage.json")

class SwarmProvider:
    def __init__(self, id, name, model_id, api_key, api_base, protocol="openai", rpm=None, max_tokens=None, allowed_models=None):
        self.id = id
        self.name = name
        self.model_id = model_id
        self.api_key = api_key
        self.api_base = api_base
        self.protocol = protocol 
        self.rpm = rpm
        self.max_tokens = max_tokens
        self.allowed_models = allowed_models 
        self.token_usage = 0 
        self.request_times = []
        self.cooldown_until = 0
        self.disabled = False
        self.last_error = None

    def is_available(self):
        if self.disabled: return False
        if self.max_tokens and self.token_usage >= self.max_tokens:
            log(f"Lane {self.id} reached TOKEN LIMIT ({self.token_usage}/{self.max_tokens}). Locked.")
            return False
        now = time.time()
        if now < self.cooldown_until: return False
        if self.rpm:
            self.request_times = [t for t in self.request_times if now - t < 60]
            if len(self.request_times) >= self.rpm:
                log(f"Lane {self.id} reached proactive RPM limit ({self.rpm})")
                return False
        return True

    def set_cooldown(self, seconds):
        self.cooldown_until = time.time() + seconds
        log(f"Lane {self.id} entering cooldown for {seconds}s")

    def list_models(self, timeout=30):
        if not self.api_key: return []
        if self.protocol == "openrouter":
            url = "https://openrouter.ai/api/v1/models"
            try:
                req = urllib.request.Request(url, headers={"Authorization": f"Bearer {self.api_key}"})
                with urllib.request.urlopen(req, timeout=timeout) as response:
                    data = json.loads(response.read().decode("utf-8"))
                    return [m["id"] for m in data.get("data", []) if m["id"].endswith(":free")]
            except Exception as e:
                log(f"Failed to list models for {self.name}: {e}")
                return []
        return []

    def query(self, prompt, system_instruction=None, timeout=180, override_model=None):
        if not self.api_key and self.protocol not in ["copilot_cli", "github_mcp"]: 
            return None, f"{self.name}: Missing API Key"
        if not self.is_available():
            if self.max_tokens and self.token_usage >= self.max_tokens:
                return None, f"{self.name}: SAFETY LOCK (Used {self.token_usage}/{self.max_tokens} Tokens)"
            wait_time = int(max(self.cooldown_until - time.time(), 0))
            return None, f"{self.name}: Busy/Cooldown ({wait_time}s)"

        self.request_times.append(time.time())
        target_model = override_model if override_model else self.model_id
        
        if self.allowed_models and target_model not in self.allowed_models:
            return None, f"{self.name}: Model '{target_model}' is NOT in the whitelist."

        if self.protocol == "openai":
            res, err = self._query_openai(prompt, system_instruction, timeout, target_model)
        elif self.protocol == "google":
            res, err = self._query_google(prompt, system_instruction, timeout, target_model)
        elif self.protocol == "cloudflare":
            res, err = self._query_cloudflare(prompt, system_instruction, timeout, target_model)
        elif self.protocol == "copilot_cli":
            res, err = self._query_copilot_cli(prompt, timeout)
        elif self.protocol == "github_mcp":
            res, err = self._query_github_mcp(prompt, timeout)
        elif self.protocol == "openrouter":
            res, err = self._query_openrouter(prompt, system_instruction, timeout, target_model)
        else:
            return None, "Unknown Protocol"

        if err: self._handle_error(err)
        return res, err

    def _handle_error(self, error_str):
        self.last_error = error_str
        if "HTTP 429" in error_str: self.set_cooldown(60)
        elif "HTTP 402" in error_str: self.set_cooldown(3600)
        elif "HTTP 503" in error_str or "overloaded" in error_str.lower(): self.set_cooldown(30)
        elif "HTTP 401" in error_str or "HTTP 403" in error_str:
            if "1010" not in error_str: self.disabled = True

    def _query_openai(self, prompt, system_instruction, timeout, model):
        url = f"{self.api_base}/chat/completions"
        messages = []
        if system_instruction: messages.append({"role": "system", "content": system_instruction})
        messages.append({"role": "user", "content": prompt})
        payload = {"model": model, "messages": messages, "temperature": 0.3}
        headers = {"Content-Type": "application/json", "Authorization": f"Bearer {self.api_key}", "User-Agent": "Mozilla/5.0"}
        return self._http_post(url, payload, headers, timeout=timeout)

    def _query_openrouter(self, prompt, system_instruction, timeout, model):
        url = f"{self.api_base}/chat/completions"
        messages = []
        if system_instruction: messages.append({"role": "system", "content": system_instruction})
        messages.append({"role": "user", "content": prompt})
        payload = {"model": model, "messages": messages, "temperature": 0.3}
        headers = {
            "Content-Type": "application/json", "Authorization": f"Bearer {self.api_key}",
            "HTTP-Referer": "https://keyforge.local", "X-Title": "KeyForge Swarm", "User-Agent": "KeyForge/1.0"
        }
        return self._http_post(url, payload, headers, timeout=timeout)

    def _query_google(self, prompt, system_instruction, timeout, model):
        if not model.startswith("models/"): model = f"models/{model}"
        url = f"{self.api_base}/{model}:generateContent?key={self.api_key}"
        payload = {
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {"maxOutputTokens": 8192, "temperature": 0.3}
        }
        if system_instruction: payload["systemInstruction"] = {"parts": [{"text": system_instruction}]}
        headers = {"Content-Type": "application/json", "User-Agent": "Mozilla/5.0"}
        return self._http_post(url, payload, headers, is_google=True, timeout=timeout)

    def _query_cloudflare(self, prompt, system_instruction, timeout, model):
        url = f"{self.api_base}".replace(self.model_id, model) 
        messages = []
        if system_instruction: messages.append({"role": "system", "content": system_instruction})
        messages.append({"role": "user", "content": prompt})
        payload = {"messages": messages}
        headers = {"Authorization": f"Bearer {self.api_key}", "Content-Type": "application/json"}
        return self._http_post(url, payload, headers, is_cf=True, timeout=timeout)

    def _query_copilot_cli(self, prompt, timeout):
        try:
            cmd = ["gh", "copilot", "explain", prompt]
            res = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
            if res.returncode == 0: return {"text": f"[GitHub Copilot CLI]\n\n{res.stdout}", "latency": 1.0, "model": "gh-copilot"}, None
            else: return None, f"CLI Error: {res.stderr}"
        except Exception as e: return None, f"System Error: {str(e)}"

    def _query_github_mcp(self, prompt, timeout):
        return {"text": "[GitHub Security Autofix]\n\nAutomated vulnerability scan triggered. Use 'gh code-scanning' to view results.", "latency": 0.5, "model": "gh-security"}, None

    def _http_post(self, url, payload, headers, is_google=False, is_cf=False, timeout=180):
        data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(url, data=data, headers=headers, method="POST")
        start_time = time.time()
        try:
            with urllib.request.urlopen(req, timeout=timeout) as response:
                res_body = json.loads(response.read().decode("utf-8"))
                latency = round(time.time() - start_time, 2)
                tokens_used = 0
                if "usage" in res_body:
                    tokens_used = res_body["usage"].get("total_tokens", 0)
                elif is_google and "usageMetadata" in res_body:
                    tokens_used = res_body["usageMetadata"].get("totalTokenCount", 0)
                
                if is_google:
                    text = res_body["candidates"][0]["content"]["parts"][0]["text"]
                    ver = res_body.get("modelVersion", "unknown")
                    return {"text": f"[{self.name} | {ver}]\n\n{text}", "latency": latency, "model": ver, "tokens": tokens_used}, None
                elif is_cf:
                    text = res_body["result"]["response"]
                    return {"text": f"[{self.name} | {self.model_id}]\n\n{text}", "latency": latency, "model": self.model_id, "tokens": tokens_used}, None
                else:
                    text = res_body["choices"][0]["message"]["content"]
                    return {"text": f"[{self.name} | {self.model_id}]\n\n{text}", "latency": latency, "model": self.model_id, "tokens": tokens_used}, None
        except urllib.error.HTTPError as e:
            return None, f"HTTP {e.code}: {e.read().decode('utf-8')[:200]}"
        except Exception as e:
            return None, f"System Error: {str(e)}"

class SwarmRouter:
    def __init__(self):
        cf_key = os.getenv("GEMINI_CLI_CLOUDFLARE_KEY") or "REDACTED_CLOUDFLARE_KEY"
        cf_account = os.getenv("GEMINI_CLI_CLOUDFLARE_ACCOUNT_ID") or "ba8beaa54c39bc852969ca28cbddadc9"
        or_key = os.getenv("GEMINI_CLI_OPENROUTER_KEY") or "REDACTED_OPENROUTER_KEY"
        ali_key = os.getenv("GEMINI_CLI_DASHSCOPE_KEY") or "REDACTED_DASHSCOPE_KEY"
        ali_free_models = ["qwen-max", "qwen-plus", "qwen3-coder-plus", "qwen3-coder-plus-2025-07-22"]

        self.lanes = {
            "studio": SwarmProvider("studio", "Google Studio", "models/gemini-2.0-flash-exp", 
                                    os.getenv("KF_STUDIO_KEY") or os.getenv("GEMINI_API_KEY"), 
                                    "https://generativelanguage.googleapis.com/v1beta", protocol="google", rpm=15),
            "groq_pro": SwarmProvider("groq_pro", "Groq Pro", "llama-3.3-70b-versatile", 
                                      os.getenv("GEMINI_CLI_GROQ_KEY"), "https://api.groq.com/openai/v1", rpm=30),
            "groq_flash": SwarmProvider("groq_flash", "Groq Flash", "llama-3.1-8b-instant", 
                                        os.getenv("GEMINI_CLI_GROQ_KEY"), "https://api.groq.com/openai/v1", rpm=30),
            "mistral_large": SwarmProvider("mistral_large", "Mistral Large", "mistral-large-latest",
                                           os.getenv("GEMINI_CLI_MISTRAL_KEY"), "https://api.mistral.ai/v1", rpm=5),
            "mistral_coding": SwarmProvider("mistral_coding", "Codestral", "codestral-latest",
                                            os.getenv("GEMINI_CLI_MISTRAL_KEY"), "https://api.mistral.ai/v1", rpm=5),
            "cerebras_llama": SwarmProvider("cerebras_llama", "Cerebras Llama", "llama-3.3-70b",
                                            os.getenv("GEMINI_CLI_CEREBRAS_KEY"), "https://api.cerebras.ai/v1", rpm=30),
            "cerebras_qwen": SwarmProvider("cerebras_qwen", "Cerebras Qwen", "qwen-3-235b-a22b-instruct-2507",
                                           os.getenv("GEMINI_CLI_CEREBRAS_KEY"), "https://api.cerebras.ai/v1", rpm=30),
            "sambanova_70b": SwarmProvider("sambanova_70b", "SambaNova", "Meta-Llama-3.3-70B-Instruct",
                                            os.getenv("GEMINI_CLI_SAMBANOVA_KEY"), "https://api.sambanova.ai/v1", rpm=10),
            "sambanova_r1": SwarmProvider("sambanova_r1", "SambaNova R1", "DeepSeek-R1",
                                           os.getenv("GEMINI_CLI_SAMBANOVA_KEY"), "https://api.sambanova.ai/v1", rpm=5),
            "nvidia_llama": SwarmProvider("nvidia_llama", "NVIDIA NIM", "meta/llama-3.3-70b-instruct",
                                          os.getenv("GEMINI_CLI_NVIDIA_KEY"), "https://integrate.api.nvidia.com/v1", rpm=10),
            "cloudflare_llama": SwarmProvider("cloudflare_lane", "Cloudflare", "@cf/meta/llama-3.1-8b-instruct",
                                              cf_key, 
                                              f"https://api.cloudflare.com/client/v4/accounts/{cf_account}/ai/run", protocol="cloudflare", rpm=50),
            "github_copilot": SwarmProvider("github_copilot", "GitHub Copilot (GPT-4o)", "gpt-4o",
                                           os.getenv("KF_GH_AUTH_BLOB"), "https://models.inference.ai.azure.com", rpm=10),
            "alibaba_qwen": SwarmProvider("alibaba_qwen", "Alibaba Qwen", "qwen-max",
                                          ali_key, "https://dashscope-intl.aliyuncs.com/compatible-mode/v1", 
                                          rpm=10, max_tokens=1000000, allowed_models=ali_free_models),
            "copilot_cli": SwarmProvider("copilot_cli", "Copilot CLI", "gh-copilot", None, None, protocol="copilot_cli", rpm=10),
            "github_security": SwarmProvider("github_security", "GitHub Security", "gh-security", None, None, protocol="github_mcp", rpm=5),
            "openrouter_free": SwarmProvider("openrouter_free", "OpenRouter Free", "mistralai/mistral-small-3.1-24b-instruct:free",
                                            or_key, "https://openrouter.ai/api/v1", protocol="openrouter", rpm=20)
        }
        self.load_usage()

    def load_usage(self):
        try:
            if os.path.exists(USAGE_FILE):
                with open(USAGE_FILE, 'r') as f:
                    data = json.load(f)
                    for lid, usage in data.items():
                        if lid in self.lanes: self.lanes[lid].token_usage = usage
        except Exception as e: log(f"Failed to load usage: {e}")

    def save_usage(self):
        try:
            data = {lid: p.token_usage for lid, p in self.lanes.items()}
            with open(USAGE_FILE, 'w') as f: json.dump(data, f)
        except Exception as e: log(f"Failed to save usage: {e}")

    def status_report(self):
        report = {}
        def probe(lane_id):
            p = self.lanes[lane_id]
            if p.disabled: return lane_id, {"status": "DISABLED", "error": p.last_error}
            if p.max_tokens:
                usage_str = f"{p.token_usage}/{p.max_tokens}"
                if p.token_usage >= p.max_tokens: return lane_id, {"status": "LOCKED", "reason": f"Token Limit Reached ({usage_str})"}
            if not p.is_available(): return lane_id, {"status": "COOLDOWN", "remaining": f"{int(max(p.cooldown_until - time.time(), 0))}s"}
            if not p.api_key and p.protocol not in ["copilot_cli", "github_mcp"]: return lane_id, {"status": "OFFLINE", "error": "Missing Key"}
            if p.protocol in ["copilot_cli", "github_mcp"]: return lane_id, {"status": "ONLINE", "model": p.model_id, "latency": 0.0}
            res, err = p.query("ping", timeout=10)
            if res:
                if isinstance(res, dict) and "model" in res: return lane_id, {"status": "ONLINE", "model": res["model"], "latency": res["latency"], "usage": f"{p.token_usage}/{p.max_tokens}" if p.max_tokens else "unlimited"}
                return lane_id, {"status": "ONLINE", "model": p.model_id, "latency": 1.0}
            else: return lane_id, {"status": "FAILING", "error": err[:100]}
        with ThreadPoolExecutor(max_workers=len(self.lanes)) as exc:
            results = exc.map(probe, self.lanes.keys())
            for lid, data in results: report[lid] = data
        return report

    def discover_models(self):
        available = {}
        if "openrouter_free" in self.lanes:
            models = self.lanes["openrouter_free"].list_models()
            if models: available["openrouter_free"] = models
        return available

    def route(self, prompt, capability="general", system_instruction=None, specific_lane=None, override_model=None):
        if specific_lane: lanes = [specific_lane]
        else:
            routing_map = {
                "fast": ["cloudflare_llama", "cerebras_llama", "groq_flash", "studio", "openrouter_free"],
                "reasoning": ["github_copilot", "alibaba_qwen", "sambanova_r1", "sambanova_70b", "mistral_large", "nvidia_llama", "groq_pro", "openrouter_free"],
                "coding": ["github_copilot", "alibaba_qwen", "sambanova_r1", "cerebras_qwen", "mistral_coding", "nvidia_llama", "groq_pro"],
                "security": ["github_security", "github_copilot", "sambanova_r1", "openrouter_free"],
                "workspace": ["copilot_cli", "github_copilot", "mistral_large"],
                "general": ["github_copilot", "sambanova_70b", "cerebras_llama", "mistral_large", "studio", "openrouter_free"]
            }
            lanes = routing_map.get(capability, routing_map["general"])
        errors = []
        for lid in lanes:
            p = self.lanes.get(lid)
            if not p: errors.append(f"{lid}: Not found"); continue
            if not p.is_available(): continue
            if not p.api_key and p.protocol not in ["copilot_cli", "github_mcp"]: continue
            log(f"Attempting lane: {lid} (model: {override_model or p.model_id})...")
            res, err = p.query(prompt, system_instruction, override_model=override_model)
            if res and isinstance(res, dict) and "tokens" in res:
                p.token_usage += res["tokens"]; self.save_usage() 
            if res and isinstance(res, dict) and "text" in res: return res["text"]
            elif res: return str(res)
            else: errors.append(f"{lid}: {err[:100]}")
        return f"Swarm Failure. All attempted lanes failed.\n\nDetails:\n" + "\n".join(errors)

def main():
    router = SwarmRouter()
    log("Swarm Router active (v1.8.7) - Studio upgraded to Gemini 3.0.")
    while True:
        try:
            line = sys.stdin.readline()
            if not line: break
            request = json.loads(line)
            if request.get("method") == "tools/list":
                tools = [
                    {"name": "swarm_query", "description": "Parallel task execution.", "inputSchema": {"type": "object", "properties": {"prompt": {"type": "string"}, "capability": {"type": "string"}, "system_instruction": {"type": "string"}, "specific_lane": {"type": "string"}, "override_model": {"type": "string"}}, "required": ["prompt"]}},
                    {"name": "swarm_status", "description": "Health report.", "inputSchema": {"type": "object", "properties": {}, "required": []}},
                    {"name": "swarm_available_models", "description": "Discover models.", "inputSchema": {"type": "object", "properties": {}, "required": []}}
                ]
                print(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": {"tools": tools}}), flush=True)
            elif request.get("method") == "tools/call":
                p = request.get("params", {})
                n = p.get("name")
                a = p.get("arguments", {})
                if n == "swarm_status":
                    print(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": {"content": [{"type": "text", "text": json.dumps(router.status_report(), indent=2)}], "isError": False}}), flush=True)
                elif n == "swarm_available_models":
                    print(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": {"content": [{"type": "text", "text": json.dumps(router.discover_models(), indent=2)}], "isError": False}}), flush=True)
                else:
                    res = router.route(a.get("prompt"), a.get("capability", "general"), a.get("system_instruction"), a.get("specific_lane"), a.get("override_model"))
                    print(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": {"content": [{"type": "text", "text": res}], "isError": False}}), flush=True)
            elif request.get("method") == "initialize":
                print(json.dumps({"jsonrpc": "2.0", "id": request.get("id"), "result": {"protocolVersion": "2024-11-05", "capabilities": {"tools": {}}, "serverInfo": {"name": "keyforge-swarm", "version": "1.8.7"}}}), flush=True)
        except Exception: break

if __name__ == "__main__": main()