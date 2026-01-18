// apps/keyforge-ui/src/api/services/WorkerService.ts

import { ValidationResult, ScoringWeights } from "../../types";

export class WorkerService {
    private worker: Worker | null = null;
    private workerReady = false;
    private lastKeyboardData: any = null;

    constructor() {
        this.initWorker();
    }

    private initWorker() {
        if (typeof Worker !== "undefined") {
            this.worker = new Worker(new URL("../worker.ts", import.meta.url), {
                type: "module",
            });
            this.worker.onmessage = (e) => {
                const { type } = e.data;
                if (type === "READY") {
                    this.workerReady = true;
                    if (this.lastKeyboardData) {
                        this.worker?.postMessage({
                            type: "LOAD_DATA",
                            payload: this.lastKeyboardData,
                        });
                    }
                }
            };
            this.worker.postMessage({ type: "INIT" });
        }
    }

    setLastKeyboardData(data: any) {
        this.lastKeyboardData = data;
        if (this.worker && this.workerReady) {
            this.worker.postMessage({ type: "LOAD_DATA", payload: data });
        }
    }

    async validateLayout(
        layoutStr: string,
        _weights?: ScoringWeights,
    ): Promise<ValidationResult> {
        if (!this.worker) throw new Error("Worker not initialized");

        return new Promise((resolve, reject) => {
            const handler = (e: MessageEvent) => {
                const { type, payload } = e.data;
                if (type === "VALIDATION_RESULT") {
                    this.worker?.removeEventListener("message", handler);
                    resolve(payload);
                } else if (type === "ERROR") {
                    this.worker?.removeEventListener("message", handler);
                    reject(new Error(payload));
                }
            };
            if (this.worker) {
                this.worker.addEventListener("message", handler);
                this.worker.postMessage({ type: "VALIDATE", payload: { layoutStr } });
            } else {
                reject(new Error("Worker lost during validation"));
            }
        });
    }

    stopSearch() {
        if (this.worker) {
            this.worker.postMessage({ type: "STOP" });
        }
    }

    toggleLocalWorker(enabled: boolean): string {
        if (enabled) {
            if (!this.worker) this.initWorker();
            return "Web Worker Started";
        } else {
            if (this.worker) {
                this.worker.terminate();
                this.worker = null;
                this.workerReady = false;
            }
            return "Web Worker Stopped";
        }
    }
}
