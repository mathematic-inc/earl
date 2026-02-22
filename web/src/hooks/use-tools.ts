import { useCallback, useEffect, useRef, useState } from "react";
import { fetchTools } from "@/lib/api";
import type { ProviderGroup, Tool } from "@/lib/types";

function groupByProvider(tools: Tool[]): ProviderGroup[] {
  const groups = new Map<string, Tool[]>();
  for (const tool of tools) {
    const existing = groups.get(tool.provider);
    if (existing) {
      existing.push(tool);
    } else {
      groups.set(tool.provider, [tool]);
    }
  }
  return Array.from(groups.entries()).map(([provider, tools]) => ({
    provider,
    tools,
  }));
}

export type ToolsState =
  | { status: "loading" }
  | { status: "error"; error: string }
  | { status: "loaded"; tools: Tool[]; groups: ProviderGroup[] };

export function useTools() {
  const [state, setState] = useState<ToolsState>({ status: "loading" });
  const fetchedRef = useRef(false);

  const refresh = useCallback(async () => {
    setState({ status: "loading" });
    try {
      const tools = await fetchTools();
      setState({ status: "loaded", tools, groups: groupByProvider(tools) });
    } catch (err) {
      setState({
        status: "error",
        error: err instanceof Error ? err.message : "Failed to load tools",
      });
    }
  }, []);

  useEffect(() => {
    if (fetchedRef.current) {
      return;
    }
    fetchedRef.current = true;
    refresh();
  }, [refresh]);

  return { state, refresh };
}
