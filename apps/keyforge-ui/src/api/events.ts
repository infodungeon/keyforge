import { listen as tauriListen, UnlistenFn } from "@tauri-apps/api/event";

// @ts-ignore
const isTauri =
  typeof window !== "undefined" && !!(window as any).__TAURI_INTERNALS__;

export async function listen<T>(
  eventName: string,
  handler: (event: { payload: T }) => void,
): Promise<UnlistenFn> {
  if (isTauri) {
    return tauriListen<T>(eventName, handler);
  } else {
    const wrappedHandler = (e: any) => {
      handler({ payload: e.detail });
    };

    window.addEventListener(eventName, wrappedHandler as any);

    return () => {
      window.removeEventListener(eventName, wrappedHandler as any);
    };
  }
}
