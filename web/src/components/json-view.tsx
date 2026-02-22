"use client";

import { Check, Copy } from "lucide-react";
import { type JSX, useCallback, useState } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface JsonViewProps {
  className?: string;
  data: unknown;
  maxInitialDepth?: number;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function JsonString({ value }: { value: string }) {
  return <span className="text-emerald-400">&quot;{value}&quot;</span>;
}

function JsonNumber({ value }: { value: number }) {
  return <span className="text-amber-400">{String(value)}</span>;
}

function JsonBoolean({ value }: { value: boolean }) {
  return <span className="text-violet-400">{String(value)}</span>;
}

function JsonNull() {
  return <span className="text-muted-foreground">null</span>;
}

function JsonKey({ name }: { name: string }) {
  return <span className="text-blue-400">&quot;{name}&quot;</span>;
}

function Punctuation({ children }: { children: React.ReactNode }) {
  return <span className="text-muted-foreground">{children}</span>;
}

function JsonValue({
  value,
  depth,
  maxDepth,
}: {
  value: unknown;
  depth: number;
  maxDepth: number;
}): JSX.Element {
  if (value === null) {
    return <JsonNull />;
  }
  if (typeof value === "string") {
    return <JsonString value={value} />;
  }
  if (typeof value === "number") {
    return <JsonNumber value={value} />;
  }
  if (typeof value === "boolean") {
    return <JsonBoolean value={value} />;
  }
  if (Array.isArray(value)) {
    return <JsonArray depth={depth} items={value} maxDepth={maxDepth} />;
  }
  if (isPlainObject(value)) {
    return <JsonObject data={value} depth={depth} maxDepth={maxDepth} />;
  }
  // Fallback for unexpected types (undefined, functions, symbols, etc.)
  return <span className="text-muted-foreground">{String(value)}</span>;
}

function JsonArray({
  items,
  depth,
  maxDepth,
}: {
  items: unknown[];
  depth: number;
  maxDepth: number;
}) {
  const [collapsed, setCollapsed] = useState(depth >= maxDepth);

  if (items.length === 0) {
    return <Punctuation>{"[]"}</Punctuation>;
  }

  const toggle = () => {
    setCollapsed((prev) => !prev);
  };

  if (collapsed) {
    return (
      <span>
        <button
          className="cursor-pointer hover:opacity-70"
          onClick={toggle}
          type="button"
        >
          <Punctuation>[</Punctuation>
          <span className="text-muted-foreground">
            {" "}
            {items.length} item{items.length !== 1 ? "s" : ""}{" "}
          </span>
          <Punctuation>]</Punctuation>
        </button>
      </span>
    );
  }

  return (
    <span>
      <button
        className="cursor-pointer hover:opacity-70"
        onClick={toggle}
        type="button"
      >
        <Punctuation>[</Punctuation>
      </button>
      <div className="pl-4">
        {items.map((item, index) => (
          <div key={`item-${String(index)}`}>
            <JsonValue depth={depth + 1} maxDepth={maxDepth} value={item} />
            {index < items.length - 1 ? <Punctuation>,</Punctuation> : null}
          </div>
        ))}
      </div>
      <Punctuation>]</Punctuation>
    </span>
  );
}

function JsonObject({
  data,
  depth,
  maxDepth,
}: {
  data: Record<string, unknown>;
  depth: number;
  maxDepth: number;
}) {
  const entries = Object.entries(data);
  const [collapsed, setCollapsed] = useState(depth >= maxDepth);

  if (entries.length === 0) {
    return <Punctuation>{"{}"}</Punctuation>;
  }

  const toggle = () => {
    setCollapsed((prev) => !prev);
  };

  if (collapsed) {
    return (
      <span>
        <button
          className="cursor-pointer hover:opacity-70"
          onClick={toggle}
          type="button"
        >
          <Punctuation>{"{"}</Punctuation>
          <span className="text-muted-foreground">
            {" "}
            {entries.length} key{entries.length !== 1 ? "s" : ""}{" "}
          </span>
          <Punctuation>{"}"}</Punctuation>
        </button>
      </span>
    );
  }

  return (
    <span>
      <button
        className="cursor-pointer hover:opacity-70"
        onClick={toggle}
        type="button"
      >
        <Punctuation>{"{"}</Punctuation>
      </button>
      <div className="pl-4">
        {entries.map(([key, val], index) => (
          <div key={key}>
            <JsonKey name={key} />
            <Punctuation>: </Punctuation>
            <JsonValue depth={depth + 1} maxDepth={maxDepth} value={val} />
            {index < entries.length - 1 ? <Punctuation>,</Punctuation> : null}
          </div>
        ))}
      </div>
      <Punctuation>{"}"}</Punctuation>
    </span>
  );
}

export function JsonView({
  data,
  className,
  maxInitialDepth = 3,
}: JsonViewProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(() => {
    let text: string;
    if (typeof data === "string") {
      try {
        // If it's a valid JSON string, pretty-print it
        text = JSON.stringify(JSON.parse(data), null, 2);
      } catch {
        text = data;
      }
    } else {
      try {
        text = JSON.stringify(data, null, 2);
      } catch {
        text = String(data);
      }
    }

    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => {
        setCopied(false);
      }, 1500);
    });
  }, [data]);

  // Determine what to render
  let content: JSX.Element;
  let parsedData: unknown = data;

  if (typeof data === "string") {
    try {
      parsedData = JSON.parse(data) as unknown;
    } catch {
      // Not valid JSON, render as plain text
      return (
        <div className={cn("relative font-mono text-sm", className)}>
          <div className="absolute top-2 right-2">
            <Button
              aria-label="Copy to clipboard"
              onClick={handleCopy}
              size="icon-xs"
              variant="ghost"
            >
              {copied ? (
                <Check className="size-3.5 text-emerald-400" />
              ) : (
                <Copy className="size-3.5 text-muted-foreground" />
              )}
            </Button>
          </div>
          <pre className="whitespace-pre-wrap break-all text-foreground">
            {data}
          </pre>
        </div>
      );
    }
  }

  if (
    parsedData === null ||
    typeof parsedData === "string" ||
    typeof parsedData === "number" ||
    typeof parsedData === "boolean"
  ) {
    content = (
      <JsonValue depth={0} maxDepth={maxInitialDepth} value={parsedData} />
    );
  } else if (Array.isArray(parsedData) || isPlainObject(parsedData)) {
    content = (
      <JsonValue depth={0} maxDepth={maxInitialDepth} value={parsedData} />
    );
  } else {
    // Fallback for non-JSON values
    content = (
      <pre className="whitespace-pre-wrap break-all text-foreground">
        {String(parsedData)}
      </pre>
    );
  }

  return (
    <div className={cn("relative font-mono text-sm", className)}>
      <div className="absolute top-2 right-2 z-10">
        <Button
          aria-label="Copy to clipboard"
          onClick={handleCopy}
          size="icon-xs"
          variant="ghost"
        >
          {copied ? (
            <Check className="size-3.5 text-emerald-400" />
          ) : (
            <Copy className="size-3.5 text-muted-foreground" />
          )}
        </Button>
      </div>
      <div className="overflow-auto">{content}</div>
    </div>
  );
}
