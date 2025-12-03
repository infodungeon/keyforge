interface HeaderProps {
  activeTab: 'analyze' | 'optimize';
  setActiveTab: (tab: 'analyze' | 'optimize') => void;
  keyboards: string[];
  selectedKeyboard: string;
  setSelectedKeyboard: (kb: string) => void;
  isSearching: boolean;

  // New Sync Props
  onSync: () => void;
  isSyncing: boolean;
}

export function Header({
  activeTab,
  setActiveTab,
  keyboards,
  selectedKeyboard,
  setSelectedKeyboard,
  isSearching,
  onSync,
  isSyncing
}: HeaderProps) {
  return (
    <header className="flex items-center gap-4 border-b border-slate-800 bg-slate-900 px-6 py-3 shrink-0">
      {/* Logo */}
      <h1 className="text-2xl font-black tracking-tight text-white select-none">
        KEY<span className="text-blue-500">FORGE</span>
      </h1>

      {/* Keyboard Selector */}
      <div className="flex gap-2 items-center ml-8">
        <label className="text-[10px] font-bold text-slate-500 uppercase">Keyboard</label>
        <select
          className="bg-slate-800 border border-slate-700 text-xs rounded px-2 py-1 outline-none focus:border-blue-500 text-slate-200"
          value={selectedKeyboard}
          onChange={(e) => setSelectedKeyboard(e.target.value)}
          disabled={isSearching}
        >
          {keyboards.map(k => <option key={k} value={k}>{k}</option>)}
        </select>
      </div>

      <div className="flex gap-2 ml-auto">
        {/* Golden Master Sync Button */}
        <button
          onClick={onSync}
          disabled={isSyncing || isSearching}
          className="px-3 py-1 text-[10px] font-bold rounded border border-blue-700 text-blue-400 hover:bg-blue-900/30 mr-4 flex items-center gap-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isSyncing ? (
            <>
              <span className="w-2 h-2 rounded-full bg-blue-400 animate-ping" />
              SYNCING...
            </>
          ) : (
            "REFRESH DATA"
          )}
        </button>

        {/* Navigation Tabs */}
        <button
          onClick={() => setActiveTab('analyze')}
          className={`px-4 py-1 text-xs font-bold rounded-full transition ${activeTab === 'analyze' ? 'bg-blue-600 text-white' : 'bg-slate-800 text-slate-400 hover:bg-slate-700'}`}
        >
          ANALYZER
        </button>
        <button
          onClick={() => setActiveTab('optimize')}
          className={`px-4 py-1 text-xs font-bold rounded-full transition ${activeTab === 'optimize' ? 'bg-purple-600 text-white' : 'bg-slate-800 text-slate-400 hover:bg-slate-700'}`}
        >
          OPTIMIZER
        </button>
      </div>
    </header>
  );
}