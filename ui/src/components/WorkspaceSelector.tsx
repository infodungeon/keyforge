import { useState } from "react";

interface Props {
  onConfirm: (path: string) => void;
}

export function WorkspaceSelector({ onConfirm }: Props) {
  const [path, setPath] = useState("");

  return (
    <div className="fixed inset-0 bg-slate-950 flex items-center justify-center z-50">
      <div className="bg-slate-900 p-8 rounded-xl border border-slate-800 shadow-2xl w-[500px]">
        <h2 className="text-2xl font-black text-white mb-2">Welcome to KeyForge</h2>
        <p className="text-slate-400 text-sm mb-6">
          Please enter the absolute path to your KeyForge workspace root (the folder containing <code className="bg-slate-800 px-1 rounded">data/</code>).
        </p>
        
        <input 
          autoFocus
          className="w-full bg-slate-800 border border-slate-700 rounded p-3 text-slate-200 mb-4 focus:border-blue-500 outline-none font-mono text-xs"
          placeholder="/path/to/keyforge"
          value={path}
          onChange={(e) => setPath(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && path && onConfirm(path)}
        />

        <button
          onClick={() => path && onConfirm(path)}
          disabled={!path}
          className="w-full bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-bold py-3 rounded"
        >
          LOAD WORKSPACE
        </button>
      </div>
    </div>
  );
}