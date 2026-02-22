"use client";

import { formatDistanceToNow } from "date-fns";
import {
  CheckCircle2,
  ChevronDown,
  ChevronUp,
  GitCompareArrows,
  History,
  Inbox,
  Play,
  Trash2,
  XCircle,
} from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import type { HistoryEntry } from "@/lib/types";
import { cn } from "@/lib/utils";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface HistoryDrawerProps {
  entries: HistoryEntry[];
  onClear: () => void;
  onReplay: (entry: HistoryEntry) => void;
}

// ---------------------------------------------------------------------------
// Diff helpers
// ---------------------------------------------------------------------------

interface DiffLine {
  text: string;
  type: "added" | "removed" | "unchanged";
}

/**
 * Simple line-by-line diff: split each JSON string by newlines and compare.
 * Returns an array of diff lines with their type.
 */
function computeDiff(previous: string, current: string): DiffLine[] {
  const prevLines = previous.split("\n");
  const currLines = current.split("\n");
  const result: DiffLine[] = [];

  const maxLen = Math.max(prevLines.length, currLines.length);

  for (let i = 0; i < maxLen; i++) {
    const prev = i < prevLines.length ? prevLines[i] : undefined;
    const curr = i < currLines.length ? currLines[i] : undefined;

    if (prev === curr) {
      result.push({ type: "unchanged", text: curr ?? "" });
    } else {
      if (prev !== undefined) {
        result.push({ type: "removed", text: prev });
      }
      if (curr !== undefined) {
        result.push({ type: "added", text: curr });
      }
    }
  }

  return result;
}

function diffLinePrefix(type: DiffLine["type"]): string {
  if (type === "added") {
    return "+";
  }
  if (type === "removed") {
    return "-";
  }
  return " ";
}

/** Serialise a history entry's response to a comparable string. */
function serialiseResponse(entry: HistoryEntry): string {
  if (entry.error) {
    try {
      return JSON.stringify(entry.error, null, 2);
    } catch {
      return String(entry.error);
    }
  }
  if (entry.response) {
    try {
      return JSON.stringify(entry.response, null, 2);
    } catch {
      return String(entry.response);
    }
  }
  return entry.responseSummary;
}

// ---------------------------------------------------------------------------
// Status badge for each entry
// ---------------------------------------------------------------------------

function EntryStatusBadge({ status }: { status: number | null }) {
  if (status === null) {
    return (
      <Badge className="gap-1 text-[0.6rem]" variant="destructive">
        <XCircle className="size-2.5" />
        Error
      </Badge>
    );
  }

  if (status >= 200 && status < 300) {
    return (
      <Badge
        className="gap-1 bg-emerald-500/20 text-[0.6rem] text-emerald-400"
        variant="secondary"
      >
        <CheckCircle2 className="size-2.5" />
        {status}
      </Badge>
    );
  }

  return (
    <Badge
      className="gap-1 bg-amber-500/20 text-[0.6rem] text-amber-400"
      variant="secondary"
    >
      {status}
    </Badge>
  );
}

// ---------------------------------------------------------------------------
// Diff view
// ---------------------------------------------------------------------------

function DiffView({
  previous,
  current,
}: {
  previous: HistoryEntry;
  current: HistoryEntry;
}) {
  const lines = useMemo(
    () => computeDiff(serialiseResponse(previous), serialiseResponse(current)),
    [previous, current]
  );

  return (
    <div className="mt-2 max-h-40 overflow-auto rounded-md border border-border bg-muted/30 p-2 font-mono text-[0.65rem] leading-relaxed">
      {lines.map((line, i) => (
        <div
          className={cn(
            "whitespace-pre-wrap px-1",
            line.type === "added" && "bg-emerald-500/20 text-emerald-300",
            line.type === "removed" && "bg-red-500/20 text-red-300",
            line.type === "unchanged" && "text-muted-foreground"
          )}
          key={`${line.type}-${String(i)}`}
        >
          <span className="mr-2 select-none text-muted-foreground/50">
            {diffLinePrefix(line.type)}
          </span>
          {line.text}
        </div>
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Single history entry row
// ---------------------------------------------------------------------------

function HistoryEntryRow({
  entry,
  previousEntry,
  onReplay,
}: {
  entry: HistoryEntry;
  previousEntry: HistoryEntry | undefined;
  onReplay: (entry: HistoryEntry) => void;
}) {
  const [showDiff, setShowDiff] = useState(false);

  const relativeTime = useMemo(
    () => formatDistanceToNow(entry.timestamp, { addSuffix: true }),
    [entry.timestamp]
  );

  const handleClick = useCallback(() => {
    onReplay(entry);
  }, [entry, onReplay]);

  const handleReplay = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onReplay(entry);
    },
    [entry, onReplay]
  );

  const handleToggleDiff = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setShowDiff((prev) => !prev);
  }, []);

  return (
    <button
      className="group w-full cursor-pointer border-border/50 border-b last:border-b-0"
      onClick={handleClick}
      type="button"
    >
      <div className="flex w-full items-center gap-2 px-3 py-2 text-left text-xs transition-colors group-hover:bg-muted/50">
        {/* Command name */}
        <span className="flex-1 truncate font-medium font-mono text-foreground">
          {entry.command}
        </span>

        {/* Status badge */}
        <EntryStatusBadge status={entry.status} />

        {/* Relative timestamp */}
        <span className="shrink-0 text-[0.6rem] text-muted-foreground">
          {relativeTime}
        </span>

        {/* Actions */}
        <span className="flex shrink-0 items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
          {previousEntry && (
            <button
              aria-label="Toggle diff"
              className="inline-flex size-5 items-center justify-center rounded-sm hover:bg-muted"
              onClick={handleToggleDiff}
              type="button"
            >
              <GitCompareArrows className="size-3" />
            </button>
          )}
          <button
            aria-label="Replay"
            className="inline-flex size-5 items-center justify-center rounded-sm hover:bg-muted"
            onClick={handleReplay}
            type="button"
          >
            <Play className="size-3" />
          </button>
        </span>
      </div>

      {/* Diff view */}
      {showDiff && previousEntry && (
        <div className="px-3 pb-2">
          <DiffView current={entry} previous={previousEntry} />
        </div>
      )}
    </button>
  );
}

// ---------------------------------------------------------------------------
// Main export
// ---------------------------------------------------------------------------

export function HistoryDrawer({
  entries,
  onReplay,
  onClear,
}: HistoryDrawerProps) {
  const [open, setOpen] = useState(false);

  /**
   * Build a map from command name to the previous entry of the same command,
   * so we can show diffs between consecutive executions of the same command.
   */
  const previousEntryMap = useMemo(() => {
    const map = new Map<string, HistoryEntry>();
    const result = new Map<string, HistoryEntry>();

    // entries are newest-first; iterate in reverse for chronological order
    for (let i = entries.length - 1; i >= 0; i--) {
      const entry = entries[i];
      if (!entry) {
        continue;
      }
      const prev = map.get(entry.command);
      if (prev) {
        result.set(entry.id, prev);
      }
      map.set(entry.command, entry);
    }

    return result;
  }, [entries]);

  return (
    <Collapsible onOpenChange={setOpen} open={open}>
      <div className="border-border border-t bg-card">
        {/* Toggle trigger */}
        <CollapsibleTrigger className="flex w-full items-center gap-2 px-3 py-2 font-medium text-muted-foreground text-xs transition-colors hover:text-foreground">
          <History className="size-3.5" />
          <span>History</span>
          {entries.length > 0 && (
            <Badge className="ml-1 text-[0.6rem]" variant="secondary">
              {entries.length}
            </Badge>
          )}
          <span className="flex-1" />
          {open ? (
            <ChevronDown className="size-3.5" />
          ) : (
            <ChevronUp className="size-3.5" />
          )}
        </CollapsibleTrigger>

        {/* Collapsible content */}
        <CollapsibleContent>
          {entries.length === 0 ? (
            /* Empty state */
            <div className="flex flex-col items-center gap-2 px-3 py-6 text-muted-foreground text-xs">
              <Inbox className="size-5" />
              <span>No requests yet</span>
            </div>
          ) : (
            <div className="relative max-h-[200px] overflow-y-auto">
              {/* Entry list */}
              <div>
                {entries.map((entry) => (
                  <HistoryEntryRow
                    entry={entry}
                    key={entry.id}
                    onReplay={onReplay}
                    previousEntry={previousEntryMap.get(entry.id)}
                  />
                ))}
              </div>

              {/* Clear history — sticky at bottom */}
              <div className="sticky bottom-0 flex justify-center border-border/50 border-t bg-card px-3 py-1.5">
                <Button
                  className="text-muted-foreground hover:text-destructive"
                  onClick={onClear}
                  size="sm"
                  variant="ghost"
                >
                  <Trash2 className="size-3" />
                  Clear history
                </Button>
              </div>
            </div>
          )}
        </CollapsibleContent>
      </div>
    </Collapsible>
  );
}
