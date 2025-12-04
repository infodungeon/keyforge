import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
    Type, Globe, Command, MousePointer2,
    Lightbulb, Layers, Zap,
    Search, ChevronRight, X, LucideIcon, Keyboard
} from "lucide-react";
import { ContextControls } from "./ContextControls"; // Self-contained now

interface Props {
    onInsert: (token: string) => void;
}

const ICON_MAP: Record<string, LucideIcon> = {
    "Type": Type,
    "Globe": Globe,
    "Command": Command,
    "MousePointer2": MousePointer2,
    "Lightbulb": Lightbulb,
    "Zap": Zap,
    "Layers": Layers
};

interface TabDef { id: string; label: string; icon: string; }
interface CategoryGroup { label: string; items: string[]; }
interface CategoryData { tabs: TabDef[]; categories: Record<string, CategoryGroup[]>; }

export function KeyPicker({ onInsert }: Props) {
    const [data, setData] = useState<CategoryData | null>(null);
    const [activeTab, setActiveTab] = useState('basic');
    const [search, setSearch] = useState('');
    const [error, setError] = useState('');

    useEffect(() => {
        invoke<CategoryData>("cmd_get_ui_categories")
            .then(setData)
            .catch(() => setError("Failed to load key data."));
    }, []);

    if (error) return <div className="w-80 p-8 text-red-400 text-xs">{error}</div>;
    if (!data) return <div className="w-80 p-8 text-slate-500 text-xs animate-pulse">Loading...</div>;

    const allKeys = Object.entries(data.categories).flatMap(([_, groups]) => groups.flatMap(g => g.items.map(k => ({ label: k, group: g.label }))));
    const filtered = search ? allKeys.filter(k => k.label.toLowerCase().includes(search.toLowerCase())) : [];

    return (
        <div className="w-80 bg-slate-900 border-l border-slate-800 flex flex-col h-full shrink-0">
            {/* 1. HEADER */}
            <div className="p-4 border-b border-slate-800 flex justify-between items-center bg-slate-950/30">
                <h3 className="text-xs font-bold text-slate-400 uppercase flex items-center gap-2">
                    <Keyboard size={14} /> Layout
                </h3>
            </div>

            {/* 2. CONTEXT CONTROLS (Self-managed) */}
            <ContextControls />

            {/* 3. SEARCH */}
            <div className="p-4 border-b border-slate-800 bg-slate-900/50">
                <div className="relative">
                    <Search className="absolute left-3 top-2.5 text-slate-500" size={14} />
                    <input
                        className="w-full bg-slate-950 border border-slate-700 rounded-lg py-2 pl-9 pr-8 text-xs text-slate-200 focus:border-purple-500 outline-none placeholder:text-slate-600"
                        placeholder="Search keycodes..."
                        value={search}
                        onChange={e => setSearch(e.target.value)}
                    />
                    {search && <button onClick={() => setSearch('')} className="absolute right-2 top-2 text-slate-500 hover:text-white"><X size={14} /></button>}
                </div>
            </div>

            {/* 4. CONTENT */}
            {search ? (
                <div className="flex-1 overflow-y-auto p-4 custom-scrollbar">
                    <div className="grid grid-cols-3 gap-2">
                        {filtered.map((k, i) => (
                            <button key={i} onClick={() => onInsert(k.label)} className="p-2 bg-slate-800 hover:bg-purple-600 hover:text-white rounded text-[10px] font-mono border border-slate-700 transition-colors relative hover:z-10">
                                {k.label}
                            </button>
                        ))}
                    </div>
                </div>
            ) : (
                <>
                    <div className="flex flex-wrap gap-1 p-2 bg-slate-900 border-b border-slate-800">
                        {data.tabs.map(tab => {
                            const Icon = ICON_MAP[tab.icon] || Type;
                            return (
                                <button key={tab.id} onClick={() => setActiveTab(tab.id)} className={`flex-1 min-w-[3rem] py-2 rounded-lg flex flex-col items-center justify-center gap-1 transition-all ${activeTab === tab.id ? 'bg-slate-800 text-purple-400 border border-slate-700' : 'text-slate-500 hover:bg-slate-800/50 hover:text-slate-300'}`}>
                                    <Icon size={16} />
                                    <span className="text-[9px] font-bold uppercase">{tab.label}</span>
                                </button>
                            );
                        })}
                    </div>
                    <div className="flex-1 overflow-y-auto p-4 custom-scrollbar">
                        {data.categories[activeTab]?.map((group, idx) => (
                            <div key={idx} className="mb-6">
                                <h4 className="text-[10px] font-bold text-slate-500 uppercase mb-2 flex items-center gap-2">
                                    <ChevronRight size={10} /> {group.label}
                                </h4>
                                <div className="grid grid-cols-4 gap-2">
                                    {group.items.map((key) => (
                                        <button
                                            key={key}
                                            onClick={() => onInsert(key)}
                                            className="h-10 px-1 bg-slate-800/50 hover:bg-slate-700 border border-slate-700/50 hover:border-purple-500/50 rounded flex items-center justify-center text-[10px] font-bold text-slate-300 transition-all hover:scale-105 hover:shadow-lg active:scale-95 relative hover:z-10"
                                            title={`Insert ${key}`}
                                        >
                                            {key}
                                        </button>
                                    ))}
                                </div>
                            </div>
                        ))}
                    </div>
                </>
            )}
        </div>
    );
}