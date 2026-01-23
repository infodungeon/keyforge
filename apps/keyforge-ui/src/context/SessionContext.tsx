import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
  useRef,
} from "react";
import { useBackend } from "./BackendContext";
import { formatForDisplay } from "../utils";
import { useLibrary } from "./LibraryContext";
import { useToast } from "./ToastContext";
import { useSystem } from "./SystemContext";

interface SessionContextType {
  layoutName: string;
  layoutString: string;
  setLayoutName: (n: string) => void;
  updateLayoutString: (s: string) => void;
  loadLayoutPreset: (name: string) => void;

  activeJobId: string | null;
  startJob: (id: string) => void;
  stopJob: () => void;

  selectedKeyIndex: number | null;
  setSelectedKeyIndex: (i: number | null) => void;

  isDatasetLoaded: boolean;
}

const SessionContext = createContext<SessionContextType | undefined>(undefined);

export function SessionProvider({ children }: { children: ReactNode }) {
  const {
    selectedKeyboard,
    selectedCorpus,
    selectedCostMatrix,
    libraryVersion,
    selectedExtras,
  } = useLibrary();

  const { hiveUrl, hiveSecret } = useSystem();
  const { addToast } = useToast();
  const backend = useBackend();

  const [layoutName, setLayoutName] = useState("Custom");
  const [layoutString, setLayoutString] = useState("");
  const [selectedKeyIndex, setSelectedKeyIndex] = useState<number | null>(null);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const [isDatasetLoaded, setIsDatasetLoaded] = useState(false);

  const isMounted = useRef(false);
  const pollTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    isMounted.current = true;
    return () => {
      isMounted.current = false;
      if (pollTimeoutRef.current) clearTimeout(pollTimeoutRef.current);
    };
  }, []);

  useEffect(() => {
    const poll = async () => {
      if (!activeJobId || !isMounted.current) return;

      try {
        const update = await backend.pollHiveStatus(
          hiveUrl,
          hiveSecret,
          activeJobId,
        );

        if (update.best_layout && isMounted.current) {
          const displayStr = formatForDisplay(update.best_layout);
          setLayoutString((prev) => (prev !== displayStr ? displayStr : prev));
        }
      } catch (e) {
        console.error("Job polling error:", e);
      }

      if (isMounted.current && activeJobId) {
        pollTimeoutRef.current = window.setTimeout(poll, 1500);
      }
    };

    if (activeJobId) {
      poll();
    } else {
      if (pollTimeoutRef.current) {
        clearTimeout(pollTimeoutRef.current);
        pollTimeoutRef.current = null;
      }
    }
  }, [activeJobId, hiveUrl, hiveSecret, backend]);

  const lastSyncParams = useRef<string>("");

  useEffect(() => {
    if (!selectedKeyboard || !selectedCorpus || !selectedCostMatrix) return;

    const syncKey = `${selectedKeyboard}:${selectedCorpus}:${selectedCostMatrix}:${libraryVersion}`;
    if (lastSyncParams.current === syncKey) return;
    lastSyncParams.current = syncKey;

    const syncSession = async () => {
      if (isMounted.current) setIsDatasetLoaded(false);

      try {
        setLayoutString("");

        await backend.loadDataset(
          selectedKeyboard,
          selectedCorpus,
          selectedCostMatrix,
          selectedExtras,
          hiveUrl,
        );

        if (!isMounted.current) return;
        setIsDatasetLoaded(true);

        const layouts = await backend.getAllLayoutsScoped(
          selectedKeyboard,
          hiveUrl,
        );
        const preferred = layouts["Colemak-DH"]
          ? "Colemak-DH"
          : layouts["Qwerty"]
            ? "Qwerty"
            : Object.keys(layouts)[0] || "Custom";
        const qmkStr = layouts[preferred] || "";

        if (isMounted.current) {
          setLayoutName(preferred);
          setLayoutString(formatForDisplay(qmkStr));
        }
        setSelectedKeyIndex(null);
      } catch (e) {
        console.error("Session Sync Failed:", e);
        if (isMounted.current) {
          setIsDatasetLoaded(true);

          setLayoutName("Custom");
          addToast("error", `Failed to load keyboard: ${e}`);
        }
      }
    };

    syncSession();
  }, [
    selectedKeyboard,
    selectedCorpus,
    selectedCostMatrix,
    selectedExtras,
    libraryVersion,
    backend,
    hiveUrl,
    addToast,
  ]);

  const updateLayoutString = (val: string) => {
    if (!isDatasetLoaded) return;
    setLayoutName("Custom");
    setLayoutString(val);
  };

  const loadLayoutPreset = async (name: string) => {
    if (!isDatasetLoaded) return;
    setLayoutName(name);
    setSelectedKeyIndex(null);
    const layouts = await backend.getAllLayoutsScoped(
      selectedKeyboard,
      hiveUrl,
    );
    if (layouts[name]) {
      setLayoutString(formatForDisplay(layouts[name]));
    }
  };

  const startJob = (id: string) => setActiveJobId(id);
  const stopJob = () => {
    setActiveJobId(null);
    backend.stopSearch().catch((e) => console.error("Stop failed:", e));
  };

  return (
    <SessionContext.Provider
      value={{
        layoutName,
        layoutString,
        setLayoutName,
        updateLayoutString,
        loadLayoutPreset,
        activeJobId,
        startJob,
        stopJob,
        selectedKeyIndex,
        setSelectedKeyIndex,
        isDatasetLoaded,
      }}
    >
      {children}
    </SessionContext.Provider>
  );
}

export const useSession = () => {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used within SessionProvider");
  return ctx;
};
