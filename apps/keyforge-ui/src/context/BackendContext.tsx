import { createContext, useContext, ReactNode, useMemo } from "react";
import { BackendClient } from "../api/client";
import { TauriClient } from "../api/tauri";
import { WebClient } from "../api/web";

// Detect Tauri environment
// @ts-ignore
const isTauri = !!window.__TAURI_INTERNALS__;

const BackendContext = createContext<BackendClient | undefined>(undefined);

export function BackendProvider({ children }: { children: ReactNode }) {
  const client = useMemo(() => {
    if (isTauri) {
      console.log("🔌 Using Tauri Backend");
      return new TauriClient();
    } else {
      console.log("🌐 Using Web Backend");
      return new WebClient();
    }
  }, []);

  return (
    <BackendContext.Provider value={client}>{children}</BackendContext.Provider>
  );
}

export const useBackend = () => {
  const ctx = useContext(BackendContext);
  if (!ctx) throw new Error("useBackend must be used within BackendProvider");
  return ctx;
};
