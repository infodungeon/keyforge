import { useState, useEffect, useRef } from "react";
import {
  Search,
  Command,
  ArrowRight,
  Zap,
  Layout,
  Settings,
  Play,
} from "lucide-react";
import { useKeyboard } from "../context/KeyboardContext";
import { useSystem } from "../context/SystemContext";
import { AppMode } from "../types";

interface Props {
  setMode: (m: AppMode) => void;
}

interface Action {
  id: string;
  label: string;
  icon: any;
  shortcut?: string;
  perform: () => void;
  group: string;
}

export function CommandPalette({ setMode }: Props) {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const { layoutName, saveUserLayout, startJob } = useKeyboard();
  const { syncData } = useSystem();

  const actions: Action[] = [
    {
      id: "nav-analyze",
      label: "Go to Analyzer",
      icon: Layout,
      group: "Navigation",
      perform: () => setMode("analyze"),
    },
    {
      id: "nav-optimize",
      label: "Go to Optimizer",
      icon: Zap,
      group: "Navigation",
      perform: () => setMode("optimize"),
    },
    {
      id: "nav-arena",
      label: "Go to Typing Arena",
      icon: Play,
      group: "Navigation",
      perform: () => setMode("arena"),
    },
    {
      id: "nav-settings",
      label: "Go to Settings",
      icon: Settings,
      group: "Navigation",
      perform: () => setMode("settings"),
    },

    {
      id: "act-save",
      label: "Save Layout",
      icon: ArrowRight,
      shortcut: "Ctrl+S",
      group: "Actions",
      perform: () => {
        const name = prompt(
          "Save layout as:",
          layoutName === "Custom" ? "" : layoutName,
        );
        if (name) saveUserLayout(name);
      },
    },
    {
      id: "act-sync",
      label: "Sync Data",
      icon: ArrowRight,
      group: "Actions",
      perform: () => syncData(),
    },
  ];

  const filtered = actions.filter((a) =>
    a.label.toLowerCase().includes(query.toLowerCase()),
  );

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setIsOpen((prev) => !prev);
      }
      if (e.key === "Escape") setIsOpen(false);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 50);
      setQuery("");
      setSelectedIndex(0);
    }
  }, [isOpen]);

  const handleSelect = (action: Action) => {
    action.perform();
    setIsOpen(false);
  };

  const handleListKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => (i + 1) % filtered.length);
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => (i - 1 + filtered.length) % filtered.length);
    }
    if (e.key === "Enter") {
      e.preventDefault();
      if (filtered[selectedIndex]) handleSelect(filtered[selectedIndex]);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-[100] flex items-start justify-center pt-[20vh]">
      <div className="w-[600px] bg-slate-900 border border-slate-700 rounded-xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-100">
        <div className="flex items-center px-4 py-3 border-b border-slate-800">
          <Search className="text-slate-500 mr-3" size={18} />
          <input
            ref={inputRef}
            className="flex-1 bg-transparent outline-none text-slate-200 placeholder:text-slate-600"
            placeholder="Type a command or search..."
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleListKeyDown}
          />
          <div className="flex gap-1">
            <span className="text-[10px] bg-slate-800 text-slate-400 px-1.5 py-0.5 rounded border border-slate-700">
              ESC
            </span>
          </div>
        </div>

        <div className="max-h-[300px] overflow-y-auto p-2">
          {filtered.length === 0 ? (
            <div className="p-4 text-center text-slate-500 text-sm">
              No results found.
            </div>
          ) : (
            filtered.map((action, i) => (
              <div
                key={action.id}
                onClick={() => handleSelect(action)}
                className={`flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer transition-colors ${i === selectedIndex ? "bg-blue-600 text-white" : "text-slate-300 hover:bg-slate-800"}`}
              >
                <div className="flex items-center gap-3">
                  <action.icon
                    size={16}
                    className={
                      i === selectedIndex ? "text-white" : "text-slate-500"
                    }
                  />
                  <span className="text-sm font-medium">{action.label}</span>
                </div>
                {action.shortcut && (
                  <span
                    className={`text-[10px] ${i === selectedIndex ? "text-blue-200" : "text-slate-500"}`}
                  >
                    {action.shortcut}
                  </span>
                )}
              </div>
            ))
          )}
        </div>

        <div className="px-4 py-2 bg-slate-950 border-t border-slate-800 flex justify-between items-center text-[10px] text-slate-500">
          <span>KeyForge Command Palette</span>
          <div className="flex gap-2">
            <span>
              Select <kbd className="bg-slate-800 px-1 rounded">↵</kbd>
            </span>
            <span>
              Navigate <kbd className="bg-slate-800 px-1 rounded">↑↓</kbd>
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
