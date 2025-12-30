import { useState } from "react";
import { AppMode } from "./types";
import { NavRail } from "./components/NavRail";
import { StatusBar } from "./components/StatusBar";
import { ToastProvider, useToast } from "./context/ToastContext";
import { ArenaProvider } from "./context/ArenaContext";
import { LibraryProvider } from "./context/LibraryContext";
import { SessionProvider } from "./context/SessionContext";
import { AnalysisProvider, useAnalysis } from "./context/AnalysisContext";
import { SystemProvider, useSystem } from "./context/SystemContext";
import { BackendProvider, useBackend } from "./context/BackendContext";
import { useKeyboard } from "./context/KeyboardContext";
import { Button } from "./components/ui/Button";
import { Input } from "./components/ui/Input";
import { CloudOff, RefreshCw } from "lucide-react";
import { CommandPalette } from "./components/CommandPalette";

// Views
import { AnalyzeView } from "./views/AnalyzeView";
import { LayoutView } from "./views/LayoutView";
import { OptimizeView } from "./views/OptimizeView";
import { ConstructView } from "./views/ConstructView";
import { ArenaView } from "./views/ArenaView";
import { TesterView } from "./views/TesterView";
import { SettingsView } from "./views/SettingsView";

function AppContent() {
  const [mode, setMode] = useState<AppMode>("analyze");
  const [pinnedKeys, setPinnedKeys] = useState("");

  const {
    startJob,
    stopJob,
    weights,
    searchParams,
    selectedCorpus,
    selectedCostMatrix,
    keyboards,
  } = useKeyboard();

  const { activeResult } = useAnalysis();

  const {
    hiveUrl,
    setHiveUrl,
    hiveSecret,
    localWorkerEnabled,
    toggleWorker,
    isSyncing,
    syncData,
    isBootstrapping,
    bootstrapError,
    retryBootstrap,
  } = useSystem();

  const { activeJobId } = useKeyboard();
  const { addToast } = useToast();
  const backend = useBackend();

  if (keyboards.length === 0) {
    return (
      <div className="h-screen bg-[#020617] text-slate-200 font-sans flex flex-col items-center justify-center p-8">
        <div className="max-w-md w-full bg-slate-900 border border-slate-800 rounded-xl p-8 shadow-2xl">
          <div className="flex flex-col items-center text-center gap-4 mb-8">
            <div className="w-16 h-16 bg-slate-800 rounded-full flex items-center justify-center text-blue-500 mb-2">
              {isBootstrapping ? (
                <RefreshCw className="animate-spin" size={32} />
              ) : (
                <CloudOff size={32} />
              )}
            </div>
            <h1 className="text-2xl font-black text-white">
              Welcome to KeyForge
            </h1>
            <p className="text-sm text-slate-400">
              Your workspace is empty. We need to download the core assets
              (Keyboards, Corpora, Physics Models) from the Hive to get started.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="text-[10px] font-bold text-slate-500 uppercase mb-1 block">
                Hive Server URL
              </label>
              <Input
                value={hiveUrl}
                onChange={(e) => setHiveUrl(e.target.value)}
                placeholder="http://localhost:3000"
                disabled={isBootstrapping}
              />
            </div>

            {bootstrapError && (
              <div className="p-3 bg-red-900/20 border border-red-900/50 rounded text-red-400 text-xs">
                {bootstrapError}
              </div>
            )}

            <Button
              className="w-full"
              onClick={retryBootstrap}
              isLoading={isBootstrapping}
              disabled={!hiveUrl}
            >
              {isBootstrapping ? "INITIALIZING..." : "CONNECT & DOWNLOAD"}
            </Button>
          </div>
        </div>
      </div>
    );
  }

  const handleDispatch = async () => {
    if (!activeResult?.geometry || !weights || !searchParams) {
      addToast(
        "error",
        "Configuration incomplete (missing geometry, weights, or params).",
      );
      return;
    }

    try {
      const corpora = (selectedCorpus || "text/en_std").split(",").map((s) => {
        const [id, w] = s.trim().split(":");
        return {
          id: id.trim(),
          weight: w ? parseFloat(w) : 1.0,
          hash: null,
        };
      });

      const request = {
        version: 1,
        definition: {
          meta: {
            name: "Custom Job",
            author: "KeyForge UI",
            version: "1.0",
            notes: "",
            type: "ortho",
          },
          geometry: activeResult.geometry,
          layouts: {},
        },
        weights: weights,
        params: searchParams,
        pinned_keys: [], // TODO: Parse pinnedKeys string into KeyConstraint[]
        corpora: corpora,
        cost_matrix: { type: "Predefined", data: selectedCostMatrix || "cost_matrix.json" } as const,
        biometrics: [],
        parent_job_id: null,
        baseline_score: null,
        parents: [],
      };

      const jobId = await backend.dispatchJob(hiveUrl, hiveSecret, request);

      startJob(jobId);
      addToast("success", "Optimization Job Dispatched to Hive");
    } catch (e) {
      addToast("error", `Dispatch Failed: ${e}`);
    }
  };

  const renderView = () => {
    const sidebarProps = {
      hiveUrl,
      isSyncing,
      onSync: syncData,
      localWorkerEnabled,
      toggleWorker,
      pinnedKeys,
      setPinnedKeys,
    };

    switch (mode) {
      case "analyze":
        return <AnalyzeView {...sidebarProps} />;
      case "layout":
        return <LayoutView isSyncing={isSyncing} onSync={syncData} />;
      case "optimize":
        return (
          <OptimizeView
            {...sidebarProps}
            onDispatch={handleDispatch}
            onStopJob={stopJob}
          />
        );
      case "design":
        return <ConstructView />;
      case "arena":
        return <ArenaView />;
      case "test":
        return <TesterView />;
      case "settings":
        return (
          <SettingsView
            hiveUrl={hiveUrl}
            setHiveUrl={setHiveUrl}
            localWorkerEnabled={localWorkerEnabled}
            toggleWorker={toggleWorker}
          />
        );
      default:
        return <AnalyzeView {...sidebarProps} />;
    }
  };

  return (
    <div className="h-screen bg-[#020617] text-slate-200 font-sans flex flex-col overflow-hidden selection:bg-blue-500/30">
      <CommandPalette setMode={setMode} />
      <div className="flex-1 flex overflow-hidden">
        <NavRail mode={mode} setMode={setMode} />
        {renderView()}
      </div>
      <StatusBar
        hiveUrl={hiveUrl}
        hiveStatus="connected"
        localWorkerEnabled={localWorkerEnabled}
        isSyncing={isSyncing}
        activeJobId={activeJobId}
      />
    </div>
  );
}

export default function App() {
  return (
    <ToastProvider>
      <BackendProvider>
        <SystemProvider>
          <LibraryProvider>
            <SessionProvider>
              <ArenaProvider>
                <AnalysisProvider>
                  <AppContent />
                </AnalysisProvider>
              </ArenaProvider>
            </SessionProvider>
          </LibraryProvider>
        </SystemProvider>
      </BackendProvider>
    </ToastProvider>
  );
}
