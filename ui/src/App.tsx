import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppMode, JobStatusUpdate } from "./types";
import { NavRail } from "./components/NavRail";
import { StatusBar } from "./components/StatusBar";
import { KeyboardProvider, useKeyboard } from "./context/KeyboardContext";
import { formatForDisplay } from "./utils";

// Views
import { AnalyzeView } from "./views/AnalyzeView";
import { LayoutView } from "./views/LayoutView";
import { OptimizeView } from "./views/OptimizeView";
import { ConstructView } from "./views/ConstructView";
import { ArenaView } from "./views/ArenaView";
import { TesterView } from "./views/TesterView";
import { SettingsView } from "./views/SettingsView";

function AppContent() {
  const [mode, setMode] = useState<AppMode>('analyze');
  const [hiveUrl, setHiveUrl] = useState(() => localStorage.getItem("keyforge_hive_url") || "http://localhost:3000");
  const [isSyncing, setIsSyncing] = useState(false);

  const [localWorkerEnabled, setLocalWorkerEnabled] = useState(true);
  const [pinnedKeys, setPinnedKeys] = useState("");

  const {
    activeResult, layoutString, updateLayoutString,
    refreshData, activeJobId, startJob, stopJob, weights
  } = useKeyboard();

  const pollIntervalRef = useRef<number | null>(null);

  useEffect(() => localStorage.setItem("keyforge_hive_url", hiveUrl), [hiveUrl]);

  // --- Global Job Manager ---
  const handleDispatch = async () => {
    if (!activeResult?.geometry || !weights) return;
    try {
      const jobId = await invoke<string>("cmd_dispatch_job", {
        hiveUrl,
        request: {
          geometry: activeResult.geometry,
          weights,
          pinned_keys: pinnedKeys,
          corpus_name: "default"
        }
      });

      startJob(jobId);
      if (pollIntervalRef.current) clearInterval(pollIntervalRef.current);

      pollIntervalRef.current = window.setInterval(async () => {
        try {
          const update = await invoke<JobStatusUpdate>("cmd_poll_hive_status", { hiveUrl, jobId });
          if (update.best_layout) {
            const displayStr = formatForDisplay(update.best_layout);
            if (displayStr !== layoutString) {
              updateLayoutString(displayStr);
            }
          }
        } catch (e) { console.warn(e); }
      }, 1500);
    } catch (e) { alert(`Dispatch Error: ${e}`); }
  };

  const handleStopJob = () => {
    if (pollIntervalRef.current) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    stopJob();
  };

  const toggleWorker = async (enabled: boolean) => {
    setLocalWorkerEnabled(enabled);
    invoke("cmd_toggle_local_worker", { enabled, hiveUrl }).catch(console.error);
  };

  const handleSync = async () => {
    setIsSyncing(true);
    try {
      await invoke("cmd_sync_data", { hiveUrl });
      await refreshData();
    } catch (e) { alert("Sync failed"); }
    finally { setIsSyncing(false); }
  };

  const renderView = () => {
    const sidebarProps = {
      hiveUrl, isSyncing, onSync: handleSync,
      localWorkerEnabled, toggleWorker,
      pinnedKeys, setPinnedKeys
    };

    switch (mode) {
      case 'analyze':
        return <AnalyzeView {...sidebarProps} />;

      case 'layout':
        return <LayoutView isSyncing={isSyncing} onSync={handleSync} />;

      case 'optimize':
        return <OptimizeView {...sidebarProps} onDispatch={handleDispatch} onStopJob={handleStopJob} />;

      case 'design':  // FIXED
        return <ConstructView />;

      case 'arena':
        return <ArenaView />;

      case 'test':    // FIXED
        return <TesterView />;

      case 'settings':
        return <SettingsView
          hiveUrl={hiveUrl}
          setHiveUrl={setHiveUrl}
          localWorkerEnabled={localWorkerEnabled}
          toggleWorker={toggleWorker}
        />;

      default:
        return <AnalyzeView {...sidebarProps} />;
    }
  };

  return (
    <div className="h-screen bg-[#020617] text-slate-200 font-sans flex flex-col overflow-hidden selection:bg-blue-500/30">
      <div className="flex-1 flex overflow-hidden">
        <NavRail mode={mode} setMode={setMode} />
        {renderView()}
      </div>
      <StatusBar hiveUrl={hiveUrl} hiveStatus="connected" localWorkerEnabled={localWorkerEnabled} isSyncing={isSyncing} activeJobId={activeJobId} />
    </div>
  );
}

export default function App() {
  return (
    <KeyboardProvider>
      <AppContent />
    </KeyboardProvider>
  );
}