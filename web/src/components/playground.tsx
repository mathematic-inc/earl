"use client";

import { AlertTriangle, Play, Square } from "lucide-react";
import {
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import { CliImport } from "@/components/cli-import";
import { CodeExamples } from "@/components/code-examples";
import { HistoryDrawer } from "@/components/history-drawer";
import { ParamForm } from "@/components/param-form";
import {
  extractBindErrorParam,
  isWriteConfirmationRequired,
  ResponseView,
} from "@/components/response-view";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Skeleton } from "@/components/ui/skeleton";

import { useHistory } from "@/hooks/use-history";
import { ApiClientError, executeCommand, validateParams } from "@/lib/api";
import type {
  ApiError,
  ExecuteResponse,
  ExecutionState,
  HistoryEntry,
  Tool,
} from "@/lib/types";
import { cn } from "@/lib/utils";

const MAC_RE = /mac/i;

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface PlaygroundProps {
  initialParams: Record<string, string>;
  loading: boolean;
  onParamsChange: (params: Record<string, unknown>) => void;
  tool: Tool | null;
}

// ---------------------------------------------------------------------------
// Execution reducer
// ---------------------------------------------------------------------------

type ExecAction =
  | {
      type: "start";
      abortController: AbortController;
      previousResponse?: ExecuteResponse;
    }
  | { type: "success"; response: ExecuteResponse; timing: number }
  | { type: "error"; error: ApiError; timing?: number }
  | { type: "reset" };

function execReducer(
  _state: ExecutionState,
  action: ExecAction
): ExecutionState {
  switch (action.type) {
    case "start":
      return {
        status: "loading",
        abortController: action.abortController,
        previousResponse: action.previousResponse,
      };
    case "success":
      return {
        status: "success",
        response: action.response,
        timing: action.timing,
      };
    case "error":
      return {
        status: "error",
        error: action.error,
        timing: action.timing,
      };
    case "reset":
      return { status: "idle" };
    default:
      return { status: "idle" };
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Build default values from param specs + initialParams overlay. */
function buildDefaults(
  tool: Tool,
  initialParams: Record<string, string>
): Record<string, unknown> {
  const values: Record<string, unknown> = {};
  for (const p of tool.params) {
    if (p.default !== undefined) {
      values[p.name] = p.default;
    }
  }
  // Overlay URL hash params
  for (const [key, value] of Object.entries(initialParams)) {
    values[key] = value;
  }
  return values;
}

/** Handle specific error codes (bind_error, write_confirmation_required). */
function handleErrorSideEffects(
  apiError: ApiError,
  setErrors: React.Dispatch<React.SetStateAction<Record<string, string>>>,
  setWriteDialogOpen: React.Dispatch<React.SetStateAction<boolean>>
) {
  if (apiError.error.code === "bind_error") {
    const param = extractBindErrorParam(apiError.error.message);
    if (param) {
      setErrors((prev) => ({
        ...prev,
        [param]: apiError.error.message,
      }));
    }
  }
  if (isWriteConfirmationRequired(apiError)) {
    setWriteDialogOpen(true);
  }
}

/** Convert an unknown catch value into an ApiError shape. */
function toApiError(err: unknown): ApiError {
  if (err instanceof ApiClientError) {
    return { error: { code: err.code, message: err.message } };
  }
  if (err instanceof DOMException && err.name === "AbortError") {
    return { error: { code: "aborted", message: "Request cancelled" } };
  }
  if (err instanceof Error) {
    return { error: { code: "unknown", message: err.message } };
  }
  return { error: { code: "unknown", message: String(err) } };
}

// ---------------------------------------------------------------------------
// Loading skeleton
// ---------------------------------------------------------------------------

function PlaygroundSkeleton() {
  return (
    <div className="flex h-full flex-col">
      {/* Request strip skeleton */}
      <div className="shrink-0 border-border border-b">
        <div className="flex items-center gap-2 px-4 py-2">
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-4 w-12 rounded-full" />
          <Skeleton className="h-4 w-10 rounded-full" />
        </div>
        <div className="grid grid-cols-3 gap-3 px-4 pb-2">
          <div className="space-y-1">
            <Skeleton className="h-3 w-12" />
            <Skeleton className="h-7 w-full" />
          </div>
          <div className="space-y-1">
            <Skeleton className="h-3 w-16" />
            <Skeleton className="h-7 w-full" />
          </div>
          <div className="space-y-1">
            <Skeleton className="h-3 w-10" />
            <Skeleton className="h-7 w-full" />
          </div>
        </div>
        <div className="border-border/50 border-t px-4 py-1.5">
          <Skeleton className="h-7 w-20" />
        </div>
      </div>
      {/* Response pane skeleton */}
      <div className="flex flex-1 items-center justify-center">
        <Skeleton className="h-4 w-32" />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Write-confirmation dialog
// ---------------------------------------------------------------------------

function WriteConfirmDialog({
  open,
  onOpenChange,
  tool,
  args,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  tool: Tool;
  args: Record<string, unknown>;
  onConfirm: () => void;
}) {
  const argSummary = useMemo(() => {
    const entries = Object.entries(args).filter(
      ([, v]) => v !== undefined && v !== null && v !== ""
    );
    if (entries.length === 0) {
      return "No arguments";
    }
    return entries
      .map(([k, v]) => `${k}: ${typeof v === "string" ? v : JSON.stringify(v)}`)
      .join("\n");
  }, [args]);

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm Write Operation</DialogTitle>
          <DialogDescription>
            <span className="font-mono font-semibold">{tool.key}</span> is a{" "}
            <span className="font-semibold text-amber-400">write</span> command.
            This may modify data.
          </DialogDescription>
        </DialogHeader>

        <div className="rounded-md border border-border bg-muted/30 p-3">
          <pre className="max-h-32 overflow-auto whitespace-pre-wrap break-all font-mono text-[0.65rem] text-muted-foreground">
            {argSummary}
          </pre>
        </div>

        <DialogFooter>
          <Button onClick={() => onOpenChange(false)} variant="outline">
            Cancel
          </Button>
          <Button
            onClick={() => {
              onOpenChange(false);
              onConfirm();
            }}
            variant="destructive"
          >
            <AlertTriangle className="size-3" />
            Confirm &amp; Execute
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function Playground({
  tool,
  loading,
  initialParams,
  onParamsChange,
}: PlaygroundProps) {
  // ----- Form state -----
  const [formValues, setFormValues] = useState<Record<string, unknown>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});

  // ----- Execution state -----
  const [execution, dispatch] = useReducer(execReducer, { status: "idle" });

  // ----- History -----
  const { entries: historyEntries, addEntry, clearHistory } = useHistory();

  // ----- Write confirmation dialog -----
  const [writeDialogOpen, setWriteDialogOpen] = useState(false);

  // ----- Server-side validation -----
  const validateControllerRef = useRef<AbortController | null>(null);
  const validateTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // ----- Form persistence per-command -----
  const formCacheRef = useRef<Map<string, Record<string, unknown>>>(new Map());
  const prevToolKeyRef = useRef<string | null>(null);

  // ----- Ref mirror of formValues for use in effects without deps -----
  const formValuesRef = useRef(formValues);
  formValuesRef.current = formValues;

  // ----- Stale tracking: did form change after last execution? -----
  const [isStale, setIsStale] = useState(false);
  const lastExecutedValuesRef = useRef<Record<string, unknown> | null>(null);

  // ----- Container ref for keyboard shortcut -----
  const containerRef = useRef<HTMLDivElement>(null);

  // ----- Last URL from successful execution (for cURL examples) -----
  const lastUrl =
    execution.status === "success" ? execution.response.url : undefined;

  // ----- Stable ref for initialParams (only used on first mount per command) -----
  const initialParamsRef = useRef(initialParams);
  initialParamsRef.current = initialParams;

  // -----------------------------------------------------------------------
  // Initialise / switch commands: save old form, restore or build new defaults
  // -----------------------------------------------------------------------
  useEffect(() => {
    const prevKey = prevToolKeyRef.current;
    const newKey = tool?.key ?? null;

    // Save current form values for the previous command
    if (prevKey && prevKey !== newKey) {
      formCacheRef.current.set(prevKey, formValuesRef.current);
    }

    if (tool) {
      // Restore cached values or build from defaults
      const cached = formCacheRef.current.get(tool.key);
      const nextValues =
        cached ?? buildDefaults(tool, initialParamsRef.current);
      setFormValues(nextValues);
      setErrors({});
      dispatch({ type: "reset" });
      setIsStale(false);
      lastExecutedValuesRef.current = null;
    }

    prevToolKeyRef.current = newKey;
    // Only run when the tool identity changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tool?.key, tool]);

  // -----------------------------------------------------------------------
  // Propagate param changes to URL hash
  // -----------------------------------------------------------------------
  const handleFormChange = useCallback(
    (values: Record<string, unknown>) => {
      setFormValues(values);
      onParamsChange(values);
      // Mark response as stale if values differ from last execution
      if (lastExecutedValuesRef.current !== null) {
        setIsStale(true);
      }
    },
    [onParamsChange]
  );

  // -----------------------------------------------------------------------
  // Server-side validation (debounced 300ms)
  // -----------------------------------------------------------------------
  useEffect(() => {
    if (!tool) {
      return;
    }
    if (Object.keys(formValues).length === 0) {
      return;
    }

    // Cancel pending validation
    if (validateTimerRef.current) {
      clearTimeout(validateTimerRef.current);
    }
    if (validateControllerRef.current) {
      validateControllerRef.current.abort();
    }

    validateTimerRef.current = setTimeout(() => {
      const controller = new AbortController();
      validateControllerRef.current = controller;

      validateParams({ command: tool.key, args: formValues }, controller.signal)
        .then((result) => {
          if (controller.signal.aborted) {
            return;
          }
          if (!result.valid && result.missing_required) {
            const serverErrors: Record<string, string> = {};
            for (const param of result.missing_required) {
              serverErrors[param] = "Required by server";
            }
            setErrors((prev) => ({ ...prev, ...serverErrors }));
          }
        })
        .catch(() => {
          // Ignore validation errors (abort, network, etc.)
        });
    }, 300);

    return () => {
      if (validateTimerRef.current) {
        clearTimeout(validateTimerRef.current);
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tool?.key, formValues, tool]);

  // -----------------------------------------------------------------------
  // Execute command
  // -----------------------------------------------------------------------
  const doExecute = useCallback(
    async (confirmWrite = false) => {
      if (!tool) {
        return;
      }

      // If write mode and not confirmed yet, show dialog
      if (tool.mode === "write" && !confirmWrite) {
        setWriteDialogOpen(true);
        return;
      }

      // Cancel any in-flight request
      if (execution.status === "loading") {
        execution.abortController.abort();
      }

      const controller = new AbortController();
      const previousResponse =
        execution.status === "success" ? execution.response : undefined;

      dispatch({
        type: "start",
        abortController: controller,
        previousResponse,
      });

      const startTime = Date.now();

      try {
        const response = await executeCommand(
          {
            command: tool.key,
            args: formValues,
            confirm_write: confirmWrite || undefined,
          },
          controller.signal
        );

        const timing = Date.now() - startTime;
        dispatch({ type: "success", response, timing });
        lastExecutedValuesRef.current = { ...formValues };
        setIsStale(false);
        addEntry(tool.key, formValues, response);
      } catch (err: unknown) {
        if (controller.signal.aborted) {
          dispatch({
            type: "error",
            error: { error: { code: "aborted", message: "Request cancelled" } },
            timing: Date.now() - startTime,
          });
          return;
        }

        const timing = Date.now() - startTime;
        const apiError = toApiError(err);

        dispatch({ type: "error", error: apiError, timing });
        addEntry(tool.key, formValues, undefined, apiError);
        handleErrorSideEffects(apiError, setErrors, setWriteDialogOpen);
      }
    },
    [tool, formValues, execution, addEntry]
  );

  const handleExecute = useCallback(() => {
    doExecute(false);
  }, [doExecute]);

  const handleConfirmWrite = useCallback(() => {
    doExecute(true);
  }, [doExecute]);

  const handleCancel = useCallback(() => {
    if (execution.status === "loading") {
      execution.abortController.abort();
    }
  }, [execution]);

  // -----------------------------------------------------------------------
  // Cmd+Enter / Ctrl+Enter keyboard shortcut
  // -----------------------------------------------------------------------
  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return;
    }

    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        doExecute(false);
      }
    }

    container.addEventListener("keydown", onKeyDown);
    return () => container.removeEventListener("keydown", onKeyDown);
  }, [doExecute]);

  // -----------------------------------------------------------------------
  // CLI import handler
  // -----------------------------------------------------------------------
  const handleCliImport = useCallback(
    (command: string, args: Record<string, string>) => {
      // Navigate to the imported command by updating the URL hash.
      // The app.tsx hashchange listener will pick this up.
      const search = new URLSearchParams(args);
      const qs = search.toString();
      const hash = qs ? `#${command}?${qs}` : `#${command}`;
      window.location.hash = hash;

      // If the command matches the current tool, fill form directly
      if (tool && tool.key === command) {
        const nextValues = { ...formValues, ...args };
        setFormValues(nextValues);
        onParamsChange(nextValues);
      }
    },
    [tool, formValues, onParamsChange]
  );

  // -----------------------------------------------------------------------
  // History replay handler
  // -----------------------------------------------------------------------
  const handleHistoryReplay = useCallback(
    (entry: HistoryEntry) => {
      // Navigate to the command
      window.location.hash = `#${entry.command}`;

      // If same command, fill form with history entry's args
      if (tool && tool.key === entry.command) {
        setFormValues(entry.args);
        onParamsChange(entry.args);
      }

      // If the entry has a cached response, show it
      if (entry.response) {
        dispatch({
          type: "success",
          response: entry.response,
          timing: 0,
        });
      } else if (entry.error) {
        dispatch({ type: "error", error: entry.error });
      }
    },
    [tool, onParamsChange]
  );

  // -----------------------------------------------------------------------
  // Render
  // -----------------------------------------------------------------------
  if (loading) {
    return <PlaygroundSkeleton />;
  }

  if (!tool) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-muted-foreground text-sm">
        No command selected
      </div>
    );
  }

  const isExecuting = execution.status === "loading";

  return (
    <div className="flex h-full flex-col" ref={containerRef} tabIndex={-1}>
      {/* ---- Request strip (fixed at top) ---- */}
      <div className="shrink-0 border-border border-b">
        {/* Header row */}
        <div className="flex items-center gap-2 px-4 py-2">
          <code className="font-semibold text-xs">{tool.key}</code>
          <span className="rounded-full bg-muted px-1.5 py-0.5 font-medium text-[0.55rem] text-muted-foreground uppercase tracking-wider">
            {tool.protocol}
          </span>
          <span
            className={cn(
              "rounded-full px-1.5 py-0.5 font-medium text-[0.55rem]",
              tool.mode === "read"
                ? "bg-emerald-500/15 text-emerald-400"
                : "bg-amber-500/15 text-amber-400"
            )}
          >
            {tool.mode}
          </span>
          <div className="ml-auto">
            <CliImport onImport={handleCliImport} />
          </div>
        </div>

        {/* Param grid */}
        <div className="px-4 pb-2">
          <ParamForm
            autoFocus
            errors={errors}
            onChange={handleFormChange}
            onErrorsChange={setErrors}
            params={tool.params}
            values={formValues}
          />
        </div>

        {/* Action row */}
        <div className="flex items-center gap-2 border-border/50 border-t px-4 py-1.5">
          {isExecuting ? (
            <Button
              className="h-7 animate-pulse"
              onClick={handleCancel}
              size="sm"
              variant="outline"
            >
              <Square className="size-3" />
              Cancel
            </Button>
          ) : (
            <Button
              className="h-7 transition-transform duration-100 hover:brightness-110 active:scale-[0.98]"
              onClick={handleExecute}
              size="sm"
            >
              <Play className="size-3" />
              Execute
            </Button>
          )}
          <span className="ml-auto text-[0.55rem] text-muted-foreground">
            {MAC_RE.test(navigator.userAgent) ? "\u2318" : "Ctrl"}+Enter
          </span>
        </div>
      </div>

      {/* ---- Collapsible code examples ---- */}
      <div className="shrink-0 border-border/50 border-b px-4 py-1">
        <CodeExamples args={formValues} lastUrl={lastUrl} tool={tool} />
      </div>

      {/* ---- Response pane (fills remaining space) ---- */}
      <div aria-live="polite" className="min-h-0 flex-1">
        <ResponseView
          execution={execution}
          isHttp={tool.protocol === "http"}
          stale={isStale}
        />
      </div>

      {/* ---- History drawer (pinned to bottom) ---- */}
      <HistoryDrawer
        entries={historyEntries}
        onClear={clearHistory}
        onReplay={handleHistoryReplay}
      />

      {/* Write confirmation dialog */}
      <WriteConfirmDialog
        args={formValues}
        onConfirm={handleConfirmWrite}
        onOpenChange={setWriteDialogOpen}
        open={writeDialogOpen}
        tool={tool}
      />
    </div>
  );
}
