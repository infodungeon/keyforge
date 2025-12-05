// ===== keyforge/ui/src/App.tsx =====
import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppMode, JobStatusUpdate, RegisterJobRequest } from "./types";
import { NavRail } from "./components/NavRail";
import { StatusBar } from "./components/StatusBar";
import { KeyboardProvider, useKeyboard } from "./context/KeyboardContext";
import { ToastProvider, useToast } from "./context/ToastContext";
import { ArenaProvider } from "./context/ArenaContext";
import { formatForDisplay } from "./utils";
import { useLibrary } from "./context/LibraryContext"; // Need for secret

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
    refreshData, activeJobId, startJob, stopJob, weights, searchParams, selectedCorpus
  } = useKeyboard();

  const { hiveSecret } = useLibrary(); // Get Secret
  const { addToast } = useToast();

  const pollIntervalRef = useRef<number | null>(null);

  useEffect(() => localStorage.setItem("keyforge_hive_url", hiveUrl), [hiveUrl]);

  // --- Global Job Manager ---
  const handleDispatch = async () => {
    if (!activeResult?.geometry || !weights || !searchParams) {
      addToast('error', "Configuration incomplete (missing geometry, weights, or params).");
      return;
    }

    try {
      const request: RegisterJobRequest = {
        geometry: activeResult.geometry,
        weights: weights,
        params: searchParams,
        pinned_keys: pinnedKeys,
        corpus_name: selectedCorpus || "default"
      };

      const jobId = await invoke<string>("cmd_dispatch_job", {
        hiveUrl,
        hiveSecret, // Pass Secret
        request
      });

      startJob(jobId);
      addToast('success', "Optimization Job Dispatched to Hive");

      if (pollIntervalRef.current) clearInterval(pollIntervalRef.current);

      pollIntervalRef.current = window.setInterval(async () => {
        try {
          const update = await invoke<JobStatusUpdate>("cmd_poll_hive_status", {
            hiveUrl,
            hiveSecret, // Pass Secret
            jobId
          });
          if (update.best_layout) {
            const displayStr = formatForDisplay(update.best_layout);
            if (displayStr !== layoutString) {
              updateLayoutString(displayStr);
            }
          }
        } catch (e) {
          // Silent fail on polling
        }
      }, 1500);
    } catch (e) {
      addToast('error', `Dispatch Failed: ${e}`);
    }
  };

  const handleStopJob = () => {
    if (pollIntervalRef.current) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    stopJob();
    addToast('info', "Job polling stopped locally.");
  };

  const toggleWorker = async (enabled: boolean) => {
    setLocalWorkerEnabled(enabled);
    try {
      const msg = await invoke<string>("cmd_toggle_local_worker", {
        enabled,
        hiveUrl,
        hiveSecret // Pass Secret
      });
      addToast('info', msg);
    } catch (e) {
      addToast('error', `Worker Error: ${e}`);
      setLocalWorkerEnabled(!enabled); // Revert UI
    }
  };

  const handleSync = async () => {
    setIsSyncing(true);
    try {
      const stats = await invoke<{ downloaded: number, errors: string[] }>("cmd_sync_data", { hiveUrl });
      await refreshData();

      if (stats.errors.length > 0) {
        addToast('warning', `Sync completed with ${stats.errors.length} errors.`);
        console.warn(stats.errors);
      } else {
        addToast('success', `Sync Complete. Downloaded ${stats.downloaded} files.`);
      }
    } catch (e) {
      addToast('error', `Sync Failed: ${e}`);
    } finally {
      setIsSyncing(false);
    }
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
      case 'design':
        return <ConstructView />;
      case 'arena':
        return <ArenaView />;
      case 'test':
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
    <ToastProvider>
      <KeyboardProvider>
        <ArenaProvider>
          <AppContent />
        </ArenaProvider>
      </KeyboardProvider>
    </ToastProvider>
  );
}