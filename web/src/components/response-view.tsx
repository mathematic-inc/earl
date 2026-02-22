"use client";

import {
  AlertTriangle,
  Check,
  Clock,
  Copy,
  HardDrive,
  Play,
  RefreshCw,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { JsonView } from "@/components/json-view";
import { Button } from "@/components/ui/button";
import type { ApiError, ExecuteResponse, ExecutionState } from "@/lib/types";
import { cn } from "@/lib/utils";

const MAC_RE = /mac/i;

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface ResponseViewProps {
  execution: ExecutionState;
  isHttp?: boolean;
  onRefreshTools?: () => void;
  stale?: boolean;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Colour class for the HTTP status badge. */
function statusColor(status: number): string {
  if (status >= 200 && status < 300) {
    return "bg-emerald-500/20 text-emerald-400 border-emerald-500/30";
  }
  if (status >= 400 && status < 500) {
    return "bg-amber-500/20 text-amber-400 border-amber-500/30";
  }
  if (status >= 500) {
    return "bg-red-500/20 text-red-400 border-red-500/30";
  }
  return "bg-zinc-500/20 text-zinc-400 border-zinc-500/30";
}

/** Human-readable byte size. */
function formatSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Estimate the byte-size of the JSON payload. */
function estimateSize(data: unknown): number {
  try {
    return new TextEncoder().encode(JSON.stringify(data)).byteLength;
  } catch {
    return 0;
  }
}

/** Format milliseconds as a human string. */
function formatTiming(ms: number): string {
  if (ms < 1000) {
    return `${Math.round(ms)}ms`;
  }
  return `${(ms / 1000).toFixed(2)}s`;
}

const BIND_ERROR_PARAM_RE = /parameter[:\s]+(\w+)/i;

/**
 * Extract param name from a `bind_error` message.
 * Typical shape: `"Missing required parameter: foo"`
 */
function extractBindErrorParam(message: string): string | null {
  const match = message.match(BIND_ERROR_PARAM_RE);
  return match ? match[1] : null;
}

/** Whether the error signals a write-confirmation prompt. */
function isWriteConfirmationRequired(error: ApiError): boolean {
  return error.error.code === "write_confirmation_required";
}

// ---------------------------------------------------------------------------
// Tiny copy-to-clipboard hook
// ---------------------------------------------------------------------------

function useCopy() {
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const copy = useCallback(async (text: string) => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }
    timerRef.current = setTimeout(() => setCopied(false), 1500);
  }, []);

  useEffect(() => {
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  return { copied, copy };
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function CopyButton({
  text,
  label = "Copy",
  className,
}: {
  text: string;
  label?: string;
  className?: string;
}) {
  const { copied, copy } = useCopy();
  return (
    <Button
      aria-label={label}
      className={className}
      onClick={() => copy(text)}
      size="icon-xs"
      variant="ghost"
    >
      {copied ? (
        <Check className="size-3 text-emerald-400" />
      ) : (
        <Copy className="size-3 text-muted-foreground" />
      )}
    </Button>
  );
}

/** Colour-coded HTTP status badge. */
function StatusBadge({ status }: { status: number }) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full border px-2 py-0.5 font-mono font-semibold text-[0.625rem]",
        statusColor(status)
      )}
    >
      {status}
    </span>
  );
}

// ---------------------------------------------------------------------------
// Success view
// ---------------------------------------------------------------------------

function SuccessView({
  response,
  timing,
  stale,
  isHttp,
}: {
  response: ExecuteResponse;
  timing: number;
  stale?: boolean;
  isHttp?: boolean;
}) {
  const [activeTab, setActiveTab] = useState<string>("raw");
  const [bodyVisible, setBodyVisible] = useState(false);

  useEffect(() => {
    const t = setTimeout(() => setBodyVisible(true), 50);
    return () => clearTimeout(t);
  }, []);

  const size = estimateSize(response.result);

  /** Serialise tab content for the copy button. */
  const tabContentText = useCallback(
    (tab: string): string => {
      if (tab === "human") {
        return response.human_output ?? "";
      }
      const data = tab === "raw" ? response.result : response.decoded;
      try {
        return JSON.stringify(data, null, 2);
      } catch {
        return String(data);
      }
    },
    [response]
  );

  return (
    <div className={cn("flex h-full flex-col", stale && "opacity-50")}>
      {/* URL bar for HTTP commands */}
      {isHttp && response.url && (
        <div className="flex items-center gap-2 border-border border-b px-3 py-1 font-mono text-[0.6rem] text-muted-foreground">
          <span className="flex-1 truncate">{response.url}</span>
          <CopyButton label="Copy URL" text={response.url} />
        </div>
      )}

      {/* Metadata bar */}
      <div className="flex flex-wrap items-center gap-2 border-border border-b px-3 py-1.5">
        <StatusBadge status={response.status} />
        <span className="flex items-center gap-1 text-[0.6rem] text-muted-foreground">
          <Clock className="size-2.5" />
          {formatTiming(timing)}
        </span>
        {size > 0 && (
          <span className="flex items-center gap-1 text-[0.6rem] text-muted-foreground">
            <HardDrive className="size-2.5" />
            {formatSize(size)}
          </span>
        )}
        {stale && (
          <span className="rounded-full bg-amber-500/20 px-1.5 py-0.5 font-medium text-[0.55rem] text-amber-400">
            Stale
          </span>
        )}

        {/* Tab switcher + copy pushed to the right */}
        <div className="ml-auto flex items-center gap-1">
          {(["raw", "decoded", "human"] as const).map((tab) => (
            <button
              className={cn(
                "rounded-md px-2 py-0.5 text-[0.6rem] transition-colors",
                activeTab === tab
                  ? "bg-muted text-foreground"
                  : "text-muted-foreground hover:text-foreground"
              )}
              key={tab}
              onClick={() => setActiveTab(tab)}
              type="button"
            >
              {tab === "human"
                ? "Human"
                : tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
          <CopyButton
            label="Copy tab content"
            text={tabContentText(activeTab)}
          />
        </div>
      </div>

      {/* Response body */}
      <div
        className={cn(
          "flex-1 overflow-auto p-3 transition-all duration-200",
          bodyVisible ? "opacity-100" : "opacity-0"
        )}
      >
        {activeTab === "raw" && <JsonView data={response.result} />}
        {activeTab === "decoded" && <JsonView data={response.decoded} />}
        {activeTab === "human" && (
          <pre className="whitespace-pre-wrap break-all font-mono text-foreground text-xs">
            {response.human_output}
          </pre>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Error views
// ---------------------------------------------------------------------------

function ErrorView({
  error,
  timing,
  onRefreshTools,
}: {
  error: ApiError;
  timing?: number;
  onRefreshTools?: () => void;
}) {
  const code = error.error.code;
  const message = error.error.message;

  // bind_error: extract param name for highlighting
  if (code === "bind_error") {
    const param = extractBindErrorParam(message);
    return (
      <div className="flex flex-col gap-3">
        <div className="rounded-md border border-amber-500/30 bg-amber-500/10 p-3 text-xs">
          <div className="flex items-center gap-2 font-semibold text-amber-400">
            <AlertTriangle className="size-3.5" />
            Bind Error
          </div>
          <p className="mt-1 text-amber-300/80">{message}</p>
          {param && (
            <p className="mt-1.5 font-mono text-amber-400">
              Parameter: <span className="font-bold">{param}</span>
            </p>
          )}
        </div>
        {timing !== undefined && (
          <span className="flex items-center gap-1 text-muted-foreground text-xs">
            <Clock className="size-3" />
            {formatTiming(timing)}
          </span>
        )}
      </div>
    );
  }

  // write_confirmation_required
  if (isWriteConfirmationRequired(error)) {
    return (
      <div className="flex flex-col gap-3">
        <div className="rounded-md border border-amber-500/30 bg-amber-500/10 p-3 text-xs">
          <div className="flex items-center gap-2 font-semibold text-amber-400">
            <AlertTriangle className="size-3.5" />
            Write Confirmation Required
          </div>
          <p className="mt-1 text-amber-300/80">{message}</p>
        </div>
      </div>
    );
  }

  // unknown_command
  if (code === "unknown_command") {
    return (
      <div className="flex flex-col gap-3">
        <div className="rounded-md border border-red-500/30 bg-red-500/10 p-3 text-xs">
          <div className="flex items-center gap-2 font-semibold text-red-400">
            <AlertTriangle className="size-3.5" />
            Unknown Command
          </div>
          <p className="mt-1 text-red-300/80">{message}</p>
          {onRefreshTools && (
            <Button
              className="mt-2 w-fit"
              onClick={onRefreshTools}
              size="sm"
              variant="outline"
            >
              <RefreshCw className="size-3" />
              Refresh commands
            </Button>
          )}
        </div>
        {timing !== undefined && (
          <span className="flex items-center gap-1 text-muted-foreground text-xs">
            <Clock className="size-3" />
            {formatTiming(timing)}
          </span>
        )}
      </div>
    );
  }

  // execution_error (default)
  const errorText = `${code}: ${message}`;
  return (
    <div className="flex flex-col gap-3">
      <div className="rounded-md border border-red-500/30 bg-red-500/10 p-3 text-xs">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 font-semibold text-red-400">
            <AlertTriangle className="size-3.5" />
            Execution Error
          </div>
          <CopyButton label="Copy error" text={errorText} />
        </div>
        <p className="mt-1 font-mono text-red-300/80">
          <span className="font-semibold text-red-400">{code}</span>
          {": "}
          {message}
        </p>
      </div>
      {timing !== undefined && (
        <span className="flex items-center gap-1 text-muted-foreground text-xs">
          <Clock className="size-3" />
          {formatTiming(timing)}
        </span>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main export
// ---------------------------------------------------------------------------

export function ResponseView({
  execution,
  onRefreshTools,
  stale,
  isHttp,
}: ResponseViewProps) {
  if (execution.status === "idle") {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
        <Play className="size-5 opacity-30" />
        <p className="text-xs">Run to see results</p>
        <p className="text-[0.6rem] opacity-60">
          {MAC_RE.test(navigator.userAgent) ? "\u2318" : "Ctrl"}+Enter or click
          Execute above
        </p>
      </div>
    );
  }

  if (execution.status === "loading") {
    // While loading, show the previous response (dimmed) if available
    if (execution.previousResponse) {
      return (
        <div className="pointer-events-none relative h-full opacity-50">
          <SuccessView
            isHttp={isHttp}
            response={execution.previousResponse}
            timing={0}
          />
        </div>
      );
    }
    return null;
  }

  if (execution.status === "error") {
    return (
      <div
        aria-live="assertive"
        className="slide-in-from-bottom-2 fade-in animate-in duration-200"
        role="alert"
      >
        <ErrorView
          error={execution.error}
          onRefreshTools={onRefreshTools}
          timing={execution.timing}
        />
      </div>
    );
  }

  // status === "success"
  return (
    <div
      aria-live="polite"
      className="slide-in-from-bottom-2 fade-in h-full animate-in duration-200"
    >
      <SuccessView
        isHttp={isHttp}
        response={execution.response}
        stale={stale}
        timing={execution.timing}
      />
    </div>
  );
}

export { extractBindErrorParam, isWriteConfirmationRequired };
