// ui/src/api/worker.ts
// @ts-ignore
let KeyforgeEngine: any = null;
let init: any = null;

async function loadWasm() {
  try {
    // Using a variable for the path to force Vite to ignore it entirely
    const pkgPath = "./wasm-pkg/keyforge_wasm";
    // @ts-ignore
    const wasm = await import(/* @vite-ignore */ pkgPath);
    init = wasm.default;
    KeyforgeEngine = wasm.KeyforgeEngine;
  } catch (e) {
    console.warn(
      "WASM Package not found, falling back to simulated engine.",
      e,
    );
  }
}

let engine: any = null;
let isSearching = false;

self.onmessage = async (e: MessageEvent) => {
  const { type, payload } = e.data;

  switch (type) {
    case "INIT":
      await loadWasm();
      if (init && KeyforgeEngine) {
        await init();
        engine = new KeyforgeEngine();
      }
      self.postMessage({ type: "READY" });
      break;

    case "LOAD_DATA":
      if (engine) {
        engine.load_keyboard(payload.keyboardName, payload.keyboardDef);
        engine.load_keycodes(payload.keycodes);
        engine.load_corpus(payload.corpusName, payload.corpus);
        engine.load_cost_matrix(payload.costName, payload.cost);
        engine.init_session(
          payload.keyboardName,
          payload.corpusName,
          payload.costName,
          payload.weights,
          payload.params,
        );
      }
      self.postMessage({ type: "DATA_LOADED" });
      break;

    case "START":
      isSearching = true;
      runSearchLoop(payload.layoutStr);
      break;

    case "STOP":
      isSearching = false;
      break;
  }
};

async function runSearchLoop(layoutStr: string) {
  let currentLayout = layoutStr;
  let epoch = 0;

  while (isSearching) {
    try {
      await new Promise((r) => setTimeout(r, 100));
      if (!isSearching) break;

      epoch += 10;

      const update = {
        epoch,
        score: engine ? 0 : Math.random() * 100, // Placeholder
        layout: currentLayout,
        ips: 1.5 + Math.random(),
      };

      self.postMessage({ type: "UPDATE", payload: update });
    } catch (e) {
      console.error("Worker loop error:", e);
      isSearching = false;
    }
  }
}
