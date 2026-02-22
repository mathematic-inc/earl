import { useCallback, useEffect, useState } from "react";
import { CommandDocs } from "@/components/command-docs";
import { Playground } from "@/components/playground";
import { Sidebar } from "@/components/sidebar";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Toaster } from "@/components/ui/sonner";
import { useTools } from "@/hooks/use-tools";

function getHashCommand(): string | null {
  const hash = window.location.hash.slice(1);
  if (!hash) {
    return null;
  }
  const [command] = hash.split("?");
  return command || null;
}

function getHashParams(): Record<string, string> {
  const hash = window.location.hash.slice(1);
  const qIndex = hash.indexOf("?");
  if (qIndex === -1) {
    return {};
  }
  const search = new URLSearchParams(hash.slice(qIndex + 1));
  return Object.fromEntries(search.entries());
}

function setHash(command: string, params?: Record<string, unknown>) {
  let hash = `#${command}`;
  if (params && Object.keys(params).length > 0) {
    const search = new URLSearchParams();
    for (const [key, value] of Object.entries(params)) {
      if (value !== undefined && value !== null && value !== "") {
        search.set(key, String(value));
      }
    }
    const qs = search.toString();
    if (qs) {
      hash += `?${qs}`;
    }
  }
  window.history.replaceState(null, "", hash);
}

export default function App() {
  const { state: toolsState, refresh } = useTools();
  const [selectedKey, setSelectedKey] = useState<string | null>(getHashCommand);

  // Auto-select first tool if none selected and tools loaded
  useEffect(() => {
    if (
      toolsState.status === "loaded" &&
      !selectedKey &&
      toolsState.tools.length > 0
    ) {
      const hashCommand = getHashCommand();
      const key =
        hashCommand && toolsState.tools.some((t) => t.key === hashCommand)
          ? hashCommand
          : toolsState.tools[0].key;
      setSelectedKey(key);
      setHash(key);
    }
  }, [toolsState, selectedKey]);

  // Listen for hash changes (browser back/forward)
  useEffect(() => {
    function onHashChange() {
      const command = getHashCommand();
      if (command) {
        setSelectedKey(command);
      }
    }
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const handleSelectCommand = useCallback((key: string) => {
    setSelectedKey(key);
    setHash(key);
  }, []);

  const handleParamsChange = useCallback((params: Record<string, unknown>) => {
    setSelectedKey((current) => {
      if (current) {
        setHash(current, params);
      }
      return current;
    });
  }, []);

  const selectedTool =
    toolsState.status === "loaded"
      ? (toolsState.tools.find((t) => t.key === selectedKey) ?? null)
      : null;

  return (
    <div className="h-screen w-screen overflow-hidden">
      <ResizablePanelGroup className="h-full" orientation="horizontal">
        <ResizablePanel
          collapsedSize={3}
          collapsible
          defaultSize="18%"
          minSize="14%"
        >
          <Sidebar
            onSelect={handleSelectCommand}
            selectedKey={selectedKey}
            state={toolsState}
          />
        </ResizablePanel>

        <ResizableHandle />

        <ResizablePanel defaultSize="42%" minSize="25%">
          <main aria-label="Command documentation" className="h-full">
            <CommandDocs
              loading={toolsState.status === "loading"}
              onRefresh={refresh}
              tool={selectedTool}
            />
          </main>
        </ResizablePanel>

        <ResizableHandle />

        <ResizablePanel defaultSize="40%" minSize="25%">
          <aside aria-label="Command playground" className="h-full">
            <div className="dark playground h-full bg-[var(--background)] text-[var(--foreground)]">
              <div className="h-full overflow-hidden bg-background text-foreground">
                <Playground
                  initialParams={getHashParams()}
                  loading={toolsState.status === "loading"}
                  onParamsChange={handleParamsChange}
                  tool={selectedTool}
                />
              </div>
            </div>
          </aside>
        </ResizablePanel>
      </ResizablePanelGroup>
      <Toaster />
    </div>
  );
}
