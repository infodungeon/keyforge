// ===== keyforge/ui/src/main.tsx =====
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { attachConsole } from "@tauri-apps/plugin-log";

attachConsole().catch((e) => {
  console.error("Failed to attach console logger:", e);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
