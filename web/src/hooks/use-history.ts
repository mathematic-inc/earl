import { useCallback, useState } from "react";
import type { ApiError, ExecuteResponse, HistoryEntry } from "@/lib/types";

const STORAGE_KEY = "earl-history";
const MAX_ENTRIES = 20;

function loadHistory(): HistoryEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveHistory(entries: HistoryEntry[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch {
    // localStorage full or unavailable — ignore
  }
}

export function useHistory() {
  const [entries, setEntries] = useState<HistoryEntry[]>(loadHistory);

  const addEntry = useCallback(
    (
      command: string,
      args: Record<string, unknown>,
      response?: ExecuteResponse,
      error?: ApiError
    ) => {
      const entry: HistoryEntry = {
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        command,
        args,
        timestamp: Date.now(),
        status: response?.status ?? null,
        responseSummary: response
          ? JSON.stringify(response.decoded).slice(0, 200)
          : (error?.error.message ?? "Error"),
        response,
        error,
      };
      setEntries((prev) => {
        const next = [entry, ...prev].slice(0, MAX_ENTRIES);
        saveHistory(next);
        return next;
      });
    },
    []
  );

  const clearHistory = useCallback(() => {
    setEntries([]);
    localStorage.removeItem(STORAGE_KEY);
  }, []);

  return { entries, addEntry, clearHistory };
}
