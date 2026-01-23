import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import { useToast } from "./ToastContext";
import { useBackend } from "./BackendContext";

interface SystemContextType {
  // Hive Configuration
  hiveUrl: string;
  setHiveUrl: (url: string) => void;
  hiveSecret: string;
  setHiveSecret: (s: string) => void;

  // Local Worker
  localWorkerEnabled: boolean;
  toggleWorker: (enabled: boolean) => Promise<void>;

  // Sync & Bootstrap
  isSyncing: boolean;
  syncData: () => Promise<void>;
  connectionStatus: "connected" | "disconnected" | "checking";
  checkConnection: () => Promise<void>;
  isBootstrapping: boolean;
  bootstrapError: string | null;
  retryBootstrap: () => Promise<void>;
}

const SystemContext = createContext<SystemContextType | undefined>(undefined);

// Load default from build-time environment variable
// Default to http://localhost:3000 for local development if not specified
const DEFAULT_HIVE_URL =
  import.meta.env.VITE_DEFAULT_HIVE_URL || "http://localhost:3000";

export function SystemProvider({ children }: { children: ReactNode }) {
  const { addToast } = useToast();
  const backend = useBackend();

  // --- STATE ---
  const [hiveUrl, setHiveUrl] = useState(
    () => localStorage.getItem("keyforge_hive_url") || DEFAULT_HIVE_URL,
  );
  const [hiveSecret, setHiveSecret] = useState(
    () => localStorage.getItem("keyforge_hive_secret") || "",
  );
  const [localWorkerEnabled, setLocalWorkerEnabled] = useState(true);
  const [isSyncing, setIsSyncing] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<
    "connected" | "disconnected" | "checking"
  >("checking");

  const [isBootstrapping, setIsBootstrapping] = useState(false);
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);

  // Persistence
  useEffect(
    () => localStorage.setItem("keyforge_hive_url", hiveUrl),
    [hiveUrl],
  );
  useEffect(
    () => localStorage.setItem("keyforge_hive_secret", hiveSecret),
    [hiveSecret],
  );

  // --- ACTIONS ---

  const toggleWorker = async (enabled: boolean) => {
    setLocalWorkerEnabled(enabled);
    try {
      const msg = await backend.toggleLocalWorker(enabled, hiveUrl, hiveSecret);
      addToast("info", msg);
    } catch (e) {
      addToast("error", `Worker Error: ${e}`);
      setLocalWorkerEnabled(!enabled); // Revert on error
    }
  };

  const checkConnection = async () => {
    setConnectionStatus("checking");
    try {
      await backend.checkHiveHealth(hiveUrl);
      setConnectionStatus("connected");
    } catch (e) {
      setConnectionStatus("disconnected");
      // Don't toast on auto-check failure to avoid annoyance if offline
    }
  };

  useEffect(() => {
    checkConnection();
  }, [hiveUrl]);

  const syncData = async () => {
    setIsSyncing(true);
    try {
      const stats = await backend.syncData(hiveUrl);

      if (stats.errors.length > 0) {
        addToast(
          "warning",
          `Sync completed with ${stats.errors.length} errors.`,
        );
        console.warn(stats.errors);
      } else {
        addToast(
          "success",
          `Sync Complete. Downloaded ${stats.downloaded} files.`,
        );
      }
    } catch (e) {
      addToast("error", `Sync Failed: ${e}`);
    } finally {
      setIsSyncing(false);
    }
  };

  const retryBootstrap = async () => {
    setIsBootstrapping(true);
    setBootstrapError(null);
    try {
      await backend.bootstrapAssets(hiveUrl);
      addToast("success", "Workspace initialized successfully.");
      // Force page reload or trigger a library refresh would be ideal here
      window.location.reload();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setBootstrapError(`Bootstrap Failed: ${msg}`);
      addToast("error", `Bootstrap Failed: ${msg}`);
    } finally {
      setIsBootstrapping(false);
    }
  };

  // Auto-start worker on boot if enabled
  useEffect(() => {
    if (localWorkerEnabled) {
      // Small delay to ensure backend is ready
      const timer = setTimeout(() => {
        toggleWorker(true);
      }, 1000);
      return () => clearTimeout(timer);
    }
  }, []);

  return (
    <SystemContext.Provider
      value={{
        hiveUrl,
        setHiveUrl,
        hiveSecret,
        setHiveSecret,
        localWorkerEnabled,
        toggleWorker,
        isSyncing,
        syncData,
        connectionStatus,
        checkConnection,
        isBootstrapping,
        bootstrapError,
        retryBootstrap,
      }}
    >
      {children}
    </SystemContext.Provider>
  );
}

export const useSystem = () => {
  const ctx = useContext(SystemContext);
  if (!ctx) throw new Error("useSystem must be used within SystemProvider");
  return ctx;
};
