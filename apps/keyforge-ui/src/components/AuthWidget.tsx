import { useState } from "react";
import { useSystem } from "../context/SystemContext";
import { useToast } from "../context/ToastContext";
import { Button } from "./ui/Button";
import { Input } from "./ui/Input";
import { Label } from "./ui/Label";
import { User, Key, LogOut, Copy } from "lucide-react";

export function AuthWidget() {
  const { hiveUrl, hiveSecret, setHiveSecret } = useSystem();
  const { addToast } = useToast();

  const [username, setUsername] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  // Check if we are logged in (simple check: do we have a secret?)
  const isLoggedIn = !!hiveSecret && hiveSecret.startsWith("kf_");

  const handleRegister = async () => {
    if (!username.trim()) return;
    setIsLoading(true);
    try {
      const res = await fetch(`${hiveUrl}/auth/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username }),
      });

      if (res.status === 409) {
        addToast("error", "Username already taken.");
        return;
      }

      if (!res.ok) throw new Error(res.statusText);

      const data = await res.json();
      setHiveSecret(data.api_key);
      addToast("success", `Welcome, ${username}!`);
    } catch (e) {
      addToast("error", `Registration failed: ${e}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogout = () => {
    if (
      confirm(
        "Remove API Key from this device? You will need to paste it again to login.",
      )
    ) {
      setHiveSecret("");
    }
  };

  const copyKey = () => {
    navigator.clipboard.writeText(hiveSecret);
    addToast("info", "API Key copied to clipboard");
  };

  if (isLoggedIn) {
    return (
      <div className="bg-slate-900 border border-green-900/30 p-4 rounded-xl">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-bold text-green-400 flex items-center gap-2">
            <User size={16} /> Authenticated
          </h3>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleLogout}
            icon={<LogOut size={14} />}
          >
            Logout
          </Button>
        </div>

        <div className="bg-slate-950 p-3 rounded border border-slate-800 flex items-center justify-between">
          <code className="text-xs text-slate-400 font-mono">
            {hiveSecret.substring(0, 8)}...
            {hiveSecret.substring(hiveSecret.length - 4)}
          </code>
          <button onClick={copyKey} className="text-slate-500 hover:text-white">
            <Copy size={14} />
          </button>
        </div>
        <p className="text-[10px] text-slate-500 mt-2">
          Use this key in the CLI via <code>keyforge auth login</code>
        </p>
      </div>
    );
  }

  return (
    <div className="bg-slate-900 border border-slate-800 p-4 rounded-xl">
      <h3 className="text-sm font-bold text-white mb-4 flex items-center gap-2">
        <Key size={16} className="text-blue-400" /> Create Account
      </h3>
      <div className="space-y-3">
        <div>
          <Label>Username</Label>
          <Input
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="Unique handle"
          />
        </div>
        <Button
          className="w-full"
          onClick={handleRegister}
          isLoading={isLoading}
          disabled={!username}
        >
          Register
        </Button>
      </div>
    </div>
  );
}
