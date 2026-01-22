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

function ensurePlainObject(val: any, name: string) {
  if (val === null || typeof val !== "object" || Array.isArray(val)) {
    throw new Error(`${name} must be a plain object, got ${typeof val}`);
  }
  if (val instanceof Map || val instanceof Set) {
    throw new Error(`${name} must be a plain object, not Map or Set`);
  }
}

function ensureArray(val: any, name: string) {
  if (!Array.isArray(val)) {
    throw new Error(`${name} must be an array, got ${typeof val}`);
  }
}

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
        try {
          ensurePlainObject(payload.keyboardDef, "keyboardDef");
          ensureArray(payload.keycodes, "keycodes");
          ensurePlainObject(payload.corpus, "corpus");
          ensurePlainObject(payload.cost, "cost");

          engine.injectKeyboard(payload.keyboardName, payload.keyboardDef);
          engine.injectKeycodes(payload.keycodes);
          engine.injectCorpus(payload.corpusName, payload.corpus);
          engine.injectCostModel(payload.costName, payload.cost);
        } catch (err: any) {
          console.error("WASM Data Load Error:", err.message);
          self.postMessage({ type: "ERROR", payload: err.message });
          return;
        }
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
