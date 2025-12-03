import {
  BarChart3, Sliders, Settings, HelpCircle,
  Hexagon, Keyboard, PenTool, Gamepad2, CheckCircle
} from "lucide-react";
import { AppMode } from "../types";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  mode: AppMode;
  setMode: (m: AppMode) => void;
}

export function NavRail({ mode, setMode }: Props) {

  const handleHelp = async () => {
    try {
      // FIXED: Verified URL
      await invoke('plugin:opener|open', { path: 'https://keyforge.infodungeon.com' });
    } catch (e) {
      console.error("Failed to open help:", e);
    }
  };

  const NavItem = ({ id, icon: Icon, label }: { id: AppMode; icon: any; label: string }) => (
    <button
      onClick={() => setMode(id)}
      className={`p-3 rounded-xl mb-2 transition-all duration-200 group relative flex items-center justify-center
        ${mode === id
          ? "bg-purple-600 text-white shadow-lg shadow-purple-900/20"
          : "text-slate-500 hover:bg-slate-800 hover:text-slate-200"
        }`}
      title={label}
    >
      <Icon size={22} strokeWidth={mode === id ? 2.5 : 2} />
      <span className="absolute left-14 bg-slate-800 text-slate-200 text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none border border-slate-700 z-50 shadow-xl">
        {label}
      </span>
    </button>
  );

  return (
    <div className="w-16 bg-slate-900 border-r border-slate-800 flex flex-col items-center py-6 shrink-0 z-20">
      <div className="mb-8 flex flex-col items-center gap-1 group cursor-default">
        <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg shadow-inner flex items-center justify-center text-white">
          <Hexagon size={18} fill="currentColor" className="opacity-90" />
        </div>
      </div>

      <nav className="flex-1 flex flex-col w-full px-2">
        <NavItem id="analyze" icon={BarChart3} label="Analyze" />
        <NavItem id="layout" icon={Keyboard} label="Layout" />
        <NavItem id="design" icon={PenTool} label="Design" />
        <NavItem id="optimize" icon={Sliders} label="Optimize" />
        <NavItem id="arena" icon={Gamepad2} label="Arena" />
        <NavItem id="test" icon={CheckCircle} label="Test" />
        <NavItem id="settings" icon={Settings} label="Settings" />
      </nav>

      <div className="px-2">
        <button
          onClick={handleHelp}
          className="p-3 rounded-xl text-slate-500 hover:bg-slate-800 hover:text-slate-200 transition-colors"
          title="Help & Wiki"
        >
          <HelpCircle size={22} />
        </button>
      </div>
    </div>
  );
}