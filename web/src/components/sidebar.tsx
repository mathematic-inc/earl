import { ChevronRight, Search, Terminal } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "@/components/ui/badge";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { ToolsState } from "@/hooks/use-tools";
import type { ProviderGroup, Tool } from "@/lib/types";
import { cn } from "@/lib/utils";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const STORAGE_KEY = "earl-sidebar-collapsed-groups";
const COLLAPSED_WIDTH_THRESHOLD = 100; // px — below this, show icon rail

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Read persisted collapsed-groups set from localStorage */
function readPersistedGroups(): Set<string> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      return new Set(JSON.parse(raw) as string[]);
    }
  } catch {
    // ignore
  }
  return new Set();
}

/** Persist collapsed-groups set to localStorage */
function persistGroups(groups: Set<string>) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([...groups]));
  } catch {
    // ignore
  }
}

/** Case-insensitive match of query against a tool */
function matchesTool(tool: Tool, query: string): boolean {
  const q = query.toLowerCase();
  return (
    tool.title.toLowerCase().includes(q) ||
    tool.key.toLowerCase().includes(q) ||
    tool.provider.toLowerCase().includes(q)
  );
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

/** Skeleton loading state */
function SidebarSkeleton() {
  return (
    <div className="flex flex-col gap-3 p-3">
      {/* Group 1 */}
      <div className="flex flex-col gap-1.5">
        <Skeleton className="h-4 w-24" />
        <Skeleton className="h-7 w-full" />
        <Skeleton className="h-7 w-full" />
        <Skeleton className="h-7 w-full" />
      </div>
      {/* Group 2 */}
      <div className="flex flex-col gap-1.5">
        <Skeleton className="h-4 w-20" />
        <Skeleton className="h-7 w-full" />
        <Skeleton className="h-7 w-full" />
        <Skeleton className="h-7 w-full" />
        <Skeleton className="h-7 w-full" />
      </div>
    </div>
  );
}

/** Empty state when no tools loaded */
function SidebarEmpty() {
  return (
    <div className="flex flex-col items-center justify-center gap-2 p-8 text-center">
      <Terminal className="size-8 text-muted-foreground/50" />
      <p className="font-medium text-muted-foreground text-sm">
        No tools loaded
      </p>
      <p className="text-muted-foreground/70 text-xs">
        Check your configuration and reload.
      </p>
    </div>
  );
}

/** Icon rail mode (collapsed sidebar) */
function IconRail({
  groups,
  onSelect,
}: {
  groups: ProviderGroup[];
  onSelect: (key: string) => void;
}) {
  return (
    <TooltipProvider>
      <div className="flex h-full flex-col items-center gap-1 py-2">
        {groups.map((group) => (
          <Tooltip key={group.provider}>
            <TooltipTrigger
              className="flex size-8 items-center justify-center rounded-md font-semibold text-muted-foreground text-xs uppercase hover:bg-muted hover:text-foreground"
              onClick={() => {
                if (group.tools.length > 0) {
                  onSelect(group.tools[0].key);
                }
              }}
            >
              {group.provider.charAt(0)}
            </TooltipTrigger>
            <TooltipContent side="right">{group.provider}</TooltipContent>
          </Tooltip>
        ))}
      </div>
    </TooltipProvider>
  );
}

/** Single tool item in the sidebar list */
function ToolItem({
  tool,
  isActive,
  isFocused,
  onSelect,
  onMouseEnter,
  itemRef,
}: {
  tool: Tool;
  isActive: boolean;
  isFocused: boolean;
  onSelect: (key: string) => void;
  onMouseEnter: () => void;
  itemRef: (el: HTMLButtonElement | null) => void;
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        className={cn(
          "group/item relative flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-[0.8rem] transition-all duration-120 hover:translate-x-0.5",
          isActive ? "bg-accent text-accent-foreground" : "hover:bg-muted",
          isFocused && !isActive && "bg-muted/60"
        )}
        onClick={() => onSelect(tool.key)}
        onMouseEnter={onMouseEnter}
        ref={itemRef}
      >
        {/* Active indicator — left accent border */}
        <span
          className={cn(
            "absolute inset-y-1 left-0 w-0.5 rounded-full bg-primary transition-all duration-200 ease-out",
            isActive ? "opacity-100" : "opacity-0"
          )}
        />

        <span className="truncate font-medium">{tool.title}</span>
      </TooltipTrigger>
      <TooltipContent side="right">
        <p className="font-medium">{tool.title}</p>
        <p className="font-mono text-[0.65rem] opacity-70">{tool.key}</p>
      </TooltipContent>
    </Tooltip>
  );
}

/** A single provider group with collapsible content */
function ProviderGroupSection({
  group,
  filteredTools,
  isOpen,
  onToggle,
  selectedKey,
  focusedKey,
  onSelect,
  onFocusTool,
  itemRefs,
}: {
  group: ProviderGroup;
  filteredTools: Tool[];
  isOpen: boolean;
  onToggle: () => void;
  selectedKey: string | null;
  focusedKey: string | null;
  onSelect: (key: string) => void;
  onFocusTool: (key: string) => void;
  itemRefs: React.MutableRefObject<Map<string, HTMLButtonElement>>;
}) {
  return (
    <Collapsible onOpenChange={() => onToggle()} open={isOpen}>
      {/* Sticky provider header */}
      <CollapsibleTrigger className="sticky top-0 z-10 flex w-full items-center gap-1.5 bg-background/95 px-3 py-1.5 backdrop-blur-sm">
        <ChevronRight
          className={cn(
            "size-3.5 shrink-0 text-muted-foreground transition-transform duration-200",
            isOpen && "rotate-90"
          )}
        />
        <span className="flex-1 text-left font-medium text-muted-foreground text-xs uppercase tracking-wider">
          {group.provider}
        </span>
        <Badge className="h-4 px-1.5 text-[0.6rem]" variant="secondary">
          {filteredTools.length}
        </Badge>
      </CollapsibleTrigger>

      <CollapsibleContent className="overflow-hidden transition-all duration-200 ease-out data-[ending-style]:grid-rows-[0fr] data-[starting-style]:grid-rows-[0fr]">
        <div className="flex flex-col gap-0.5 px-1 pb-1">
          {filteredTools.map((tool) => (
            <ToolItem
              isActive={tool.key === selectedKey}
              isFocused={tool.key === focusedKey}
              itemRef={(el) => {
                if (el) {
                  itemRefs.current.set(tool.key, el);
                } else {
                  itemRefs.current.delete(tool.key);
                }
              }}
              key={tool.key}
              onMouseEnter={() => onFocusTool(tool.key)}
              onSelect={onSelect}
              tool={tool}
            />
          ))}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}

// ---------------------------------------------------------------------------
// Main Sidebar component
// ---------------------------------------------------------------------------

interface SidebarProps {
  onSelect: (key: string) => void;
  selectedKey: string | null;
  state: ToolsState;
}

export function Sidebar({ state, selectedKey, onSelect }: SidebarProps) {
  // --- Search filter ---
  const [search, setSearch] = useState("");

  // --- Command dialog (Cmd+K) ---
  const [cmdOpen, setCmdOpen] = useState(false);

  // --- Collapsed groups ---
  const [manuallyToggled, setManuallyToggled] =
    useState<Set<string>>(readPersistedGroups);
  const [autoExpandedProvider, setAutoExpandedProvider] = useState<
    string | null
  >(null);

  // --- Keyboard focus ---
  const [focusedKey, setFocusedKey] = useState<string | null>(null);
  const itemRefs = useRef<Map<string, HTMLButtonElement>>(new Map());
  const searchInputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // --- Collapsed width detection ---
  const [isCollapsedWidth, setIsCollapsedWidth] = useState(false);

  // Observe the container width to detect collapsed mode
  useEffect(() => {
    const el = containerRef.current;
    if (!el) {
      return;
    }
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setIsCollapsedWidth(
          entry.contentRect.width < COLLAPSED_WIDTH_THRESHOLD
        );
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // --- Derived data ---
  const groups: ProviderGroup[] = state.status === "loaded" ? state.groups : [];

  const filteredGroups = useMemo(() => {
    if (!search.trim()) {
      return groups;
    }
    return groups
      .map((g) => ({
        ...g,
        tools: g.tools.filter((t) => matchesTool(t, search)),
      }))
      .filter((g) => g.tools.length > 0);
  }, [groups, search]);

  const totalFilteredCount = useMemo(
    () => filteredGroups.reduce((sum, g) => sum + g.tools.length, 0),
    [filteredGroups]
  );

  // --- Group open/close logic ---
  const isGroupOpen = useCallback(
    (provider: string): boolean => {
      if (manuallyToggled.has(provider)) {
        return false;
      }
      if (autoExpandedProvider === provider) {
        return true;
      }
      return true;
    },
    [manuallyToggled, autoExpandedProvider]
  );

  // Build a flat list of visible (non-collapsed) tool keys for keyboard nav
  const visibleKeys = useMemo(() => {
    const keys: string[] = [];
    for (const group of filteredGroups) {
      if (isGroupOpen(group.provider)) {
        for (const tool of group.tools) {
          keys.push(tool.key);
        }
      }
    }
    return keys;
  }, [filteredGroups, isGroupOpen]);

  // When selectedKey changes, auto-expand the group containing it and
  // collapse others (unless manually toggled open)
  useEffect(() => {
    if (!selectedKey || state.status !== "loaded") {
      return;
    }
    const ownerGroup = groups.find((g) =>
      g.tools.some((t) => t.key === selectedKey)
    );
    if (ownerGroup) {
      setAutoExpandedProvider(ownerGroup.provider);
      // If the selected tool's group was manually collapsed, un-collapse it
      setManuallyToggled((prev) => {
        if (prev.has(ownerGroup.provider)) {
          const next = new Set(prev);
          next.delete(ownerGroup.provider);
          persistGroups(next);
          return next;
        }
        return prev;
      });
    }
  }, [selectedKey, state.status, groups]);

  const handleToggleGroup = useCallback((provider: string) => {
    setManuallyToggled((prev) => {
      const next = new Set(prev);
      if (next.has(provider)) {
        next.delete(provider);
      } else {
        next.add(provider);
      }
      persistGroups(next);
      return next;
    });
  }, []);

  // --- Keyboard navigation ---
  const navigateArrow = useCallback(
    (direction: "down" | "up") => {
      if (visibleKeys.length === 0) {
        return;
      }
      const currentIdx = focusedKey ? visibleKeys.indexOf(focusedKey) : -1;
      let nextIdx: number;
      if (direction === "down") {
        nextIdx =
          currentIdx === -1 || currentIdx >= visibleKeys.length - 1
            ? 0
            : currentIdx + 1;
      } else {
        nextIdx = currentIdx <= 0 ? visibleKeys.length - 1 : currentIdx - 1;
      }
      const nextKey = visibleKeys[nextIdx];
      setFocusedKey(nextKey);
      itemRefs.current.get(nextKey)?.scrollIntoView({ block: "nearest" });
    },
    [visibleKeys, focusedKey]
  );

  // Keyboard navigation on the container
  useEffect(() => {
    const el = containerRef.current;
    if (!el) {
      return;
    }
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        setSearch("");
        setFocusedKey(null);
        searchInputRef.current?.blur();
        return;
      }
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        navigateArrow(e.key === "ArrowDown" ? "down" : "up");
        return;
      }
      if (e.key === "Enter" && focusedKey) {
        e.preventDefault();
        onSelect(focusedKey);
      }
    }
    el.addEventListener("keydown", handleKeyDown);
    return () => el.removeEventListener("keydown", handleKeyDown);
  }, [navigateArrow, focusedKey, onSelect]);

  // --- Cmd+K global shortcut ---
  useEffect(() => {
    function handleGlobalKeyDown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setCmdOpen((prev) => !prev);
      }
    }
    document.addEventListener("keydown", handleGlobalKeyDown);
    return () => document.removeEventListener("keydown", handleGlobalKeyDown);
  }, []);

  // --- Render ---

  if (state.status === "error") {
    return (
      <div
        className="flex h-full items-center justify-center p-4"
        ref={containerRef}
      >
        <p className="text-destructive text-sm">{state.error}</p>
      </div>
    );
  }

  if (state.status === "loading") {
    return (
      <div className="h-full" ref={containerRef}>
        <SidebarSkeleton />
      </div>
    );
  }

  if (state.groups.length === 0) {
    return (
      <div className="h-full" ref={containerRef}>
        <SidebarEmpty />
      </div>
    );
  }

  // Icon rail mode
  if (isCollapsedWidth) {
    return (
      <div className="h-full" ref={containerRef}>
        <IconRail groups={groups} onSelect={onSelect} />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col" ref={containerRef} tabIndex={-1}>
      {/* Search filter — always visible */}
      <div className="shrink-0 border-b px-3 py-2">
        <div className="relative">
          <Search className="absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            className="h-7 pr-12 pl-7 text-xs"
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Filter commands..."
            ref={searchInputRef}
            value={search}
          />
          <kbd className="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 select-none rounded border border-border bg-muted px-1 py-0.5 font-mono text-[0.6rem] text-muted-foreground">
            {"\u2318"}K
          </kbd>
        </div>
        {search.trim() && (
          <p className="mt-1 text-[0.65rem] text-muted-foreground">
            {totalFilteredCount} result{totalFilteredCount !== 1 ? "s" : ""}
          </p>
        )}
      </div>

      {/* Scrollable tool list */}
      <ScrollArea className="min-h-0 flex-1">
        <TooltipProvider>
          <nav aria-label="Command navigation" className="flex flex-col py-1">
            {filteredGroups.map((group) => (
              <ProviderGroupSection
                filteredTools={group.tools}
                focusedKey={focusedKey}
                group={group}
                isOpen={isGroupOpen(group.provider)}
                itemRefs={itemRefs}
                key={group.provider}
                onFocusTool={setFocusedKey}
                onSelect={onSelect}
                onToggle={() => handleToggleGroup(group.provider)}
                selectedKey={selectedKey}
              />
            ))}
            {filteredGroups.length === 0 && search.trim() && (
              <div className="px-3 py-6 text-center text-muted-foreground text-xs">
                No commands matching &ldquo;{search}&rdquo;
              </div>
            )}
          </nav>
        </TooltipProvider>
      </ScrollArea>

      {/* Cmd+K command dialog */}
      <CommandDialog onOpenChange={setCmdOpen} open={cmdOpen}>
        <Command>
          <CommandInput placeholder="Search all commands..." />
          <CommandList>
            <CommandEmpty>No commands found.</CommandEmpty>
            {groups.map((group) => (
              <CommandGroup heading={group.provider} key={group.provider}>
                {group.tools.map((tool) => (
                  <CommandItem
                    key={tool.key}
                    onSelect={() => {
                      onSelect(tool.key);
                      setCmdOpen(false);
                    }}
                    value={`${tool.provider} ${tool.title} ${tool.key}`}
                  >
                    <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                      <span className="truncate font-medium text-xs">
                        {tool.title}
                      </span>
                      <span className="truncate font-mono text-[0.6rem] text-muted-foreground">
                        {tool.provider}.{tool.command}
                      </span>
                    </div>
                  </CommandItem>
                ))}
              </CommandGroup>
            ))}
          </CommandList>
        </Command>
      </CommandDialog>
    </div>
  );
}
