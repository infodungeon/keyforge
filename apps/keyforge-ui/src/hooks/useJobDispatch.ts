import { useState } from "react";
import { useBackend } from "../context/BackendContext";
import { useToast } from "../context/ToastContext";
import { useKeyboard } from "../context/KeyboardContext";
import { useAnalysis } from "../context/AnalysisContext";
import type { RegisterJobRequest, CorpusSource } from "../types";

interface PinnedConstraint {
  index: number;
  key: string;
}

/**
 * Hook for dispatching optimization jobs to the Hive.
 * Encapsulates all data transformation logic for job requests.
 */
export function useJobDispatch() {
  const [isDispatching, setIsDispatching] = useState(false);
  const backend = useBackend();
  const { addToast } = useToast();
  const { startJob, weights, searchParams, selectedCorpus, selectedCostMatrix } = useKeyboard();
  const { activeResult } = useAnalysis();

  /**
   * Parse corpus string format: "id:weight,id:weight" → structured corpus array
   * Example: "text/en_std:1.0,text/code:0.5" → [{ id: "text/en_std", weight: 1.0 }, ...]
   */
  const parseCorpora = (corpusString: string): CorpusSource[] => {
    return corpusString.split(",").map((s) => {
      const [id, w] = s.trim().split(":");
      return {
        id: id.trim(),
        weight: w ? parseFloat(w) : 1.0,
        hash: undefined,
      };
    });
  };

  /**
   * Parse pinned keys format: "index:keycode,index:keycode" → structured constraints array
   * Example: "0:a,1:b" → [{ index: 0, key: "a" }, { index: 1, key: "b" }]
   */
  const parsePinnedConstraints = (pinnedKeys: string): PinnedConstraint[] => {
    return pinnedKeys
      .split(",")
      .filter((s) => s.trim().length > 0)
      .map((s) => {
        const [idxStr, key] = s.trim().split(":");
        const index = parseInt(idxStr);
        if (isNaN(index) || !key) return null;
        return { index, key: key.trim() };
      })
      .filter((c): c is PinnedConstraint => c !== null);
  };

  /**
   * Build a complete job request DTO from current state
   */
  const buildJobRequest = (
    geometry: any,
    weights: any,
    searchParams: any,
    corpusString: string,
    pinnedKeys: string,
    costMatrix: string,
  ): RegisterJobRequest => {
    const corpora = parseCorpora(corpusString);
    const pinnedConstraints = parsePinnedConstraints(pinnedKeys);

    return {
      version: 1,
      definition: {
        meta: {
          name: "Custom Job",
          author: "KeyForge UI",
          version: "1.0",
          notes: "",
          type: "ortho",
        },
        geometry: geometry,
        layouts: {},
      },
      weights: weights,
      params: searchParams,
      pinned_keys: pinnedConstraints,
      corpora: corpora,
      cost_matrix: {
        type: "Predefined",
        data: costMatrix,
      } as const,
      biometrics: [],
      parent_job_id: null,
      baseline_score: null,
      parents: [],
    };
  };

  /**
   * Dispatch an optimization job to the Hive
   */
  const dispatchJob = async (hiveUrl: string, hiveSecret: string, pinnedKeys: string) => {
    if (!activeResult?.geometry || !weights || !searchParams) {
      addToast(
        "error",
        "Configuration incomplete (missing geometry, weights, or params).",
      );
      return;
    }

    setIsDispatching(true);

    try {
      const request = buildJobRequest(
        activeResult.geometry,
        weights,
        searchParams,
        selectedCorpus || "text/en_std",
        pinnedKeys,
        selectedCostMatrix || "cost_matrix.json",
      );

      const jobId = await backend.dispatchJob(hiveUrl, hiveSecret, request);

      startJob(jobId);
      addToast("success", "Optimization Job Dispatched to Hive");
    } catch (e) {
      addToast("error", `Dispatch Failed: ${e}`);
    } finally {
      setIsDispatching(false);
    }
  };

  return {
    dispatchJob,
    isDispatching,
  };
}
