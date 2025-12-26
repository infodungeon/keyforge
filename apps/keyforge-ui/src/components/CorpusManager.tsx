import { useState, useEffect } from "react";
import { useBackend } from "../context/BackendContext";
import { useSystem } from "../context/SystemContext";
import { useToast } from "../context/ToastContext";
import { Button } from "./ui/Button";
import { FileText, RefreshCw } from "lucide-react";

interface CorpusStats {
  name: string;
  size_bytes: number;
  path: string;
}

export function CorpusManager() {
  const [stats, setStats] = useState<CorpusStats[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { addToast } = useToast();
  const { hiveUrl } = useSystem();
  const backend = useBackend();

  const loadStats = async () => {
    setIsLoading(true);
    try {
      const data = await backend.getCorpusStats(hiveUrl);
      setStats(data);
    } catch (e) {
      addToast("error", `Failed to load corpus stats: ${e}`);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadStats();
  }, []);

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-bold text-slate-300 uppercase">
          Standard Library
        </h3>
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="ghost"
            onClick={loadStats}
            isLoading={isLoading}
            icon={<RefreshCw size={14} />}
          />
        </div>
      </div>

      <div className="bg-slate-950 border border-slate-800 rounded-lg overflow-hidden">
        <table className="w-full text-xs text-left">
          <thead className="bg-slate-900 text-slate-500 font-bold uppercase">
            <tr>
              <th className="p-3">Name</th>
              <th className="p-3 text-right">Size</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800">
            {stats.map((c) => (
              <tr
                key={c.name}
                className="group hover:bg-slate-900/50 transition-colors"
              >
                <td className="p-3 font-mono text-slate-300 flex items-center gap-2">
                  <FileText size={12} className="text-slate-600" />
                  {c.name}
                </td>
                <td className="p-3 text-right text-slate-500 font-mono">
                  {formatSize(c.size_bytes)}
                </td>
              </tr>
            ))}
            {stats.length === 0 && (
              <tr>
                <td
                  colSpan={2}
                  className="p-8 text-center text-slate-600 italic"
                >
                  No corpora found.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      <p className="text-[10px] text-slate-500 italic text-center">
        Custom corpus ingestion is disabled in this version.
      </p>
    </div>
  );
}
