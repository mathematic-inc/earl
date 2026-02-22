"use client";

import { Check, ChevronDown, Code, Copy } from "lucide-react";
import {
  useCallback,
  useDeferredValue,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { buildCliExample, buildCurlExample } from "@/lib/cli-builder";
import type { Tool } from "@/lib/types";
import { cn } from "@/lib/utils";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface CodeExamplesProps {
  args: Record<string, unknown>;
  lastUrl?: string;
  tool: Tool;
}

// ---------------------------------------------------------------------------
// Copy hook (clipboard icon -> check for 1.5s)
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
// Copy button
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

// ---------------------------------------------------------------------------
// Main export
// ---------------------------------------------------------------------------

export function CodeExamples({ tool, args, lastUrl }: CodeExamplesProps) {
  const deferredArgs = useDeferredValue(args);

  const showCurl = tool.protocol === "http";

  const cliSnippet = useMemo(
    () => buildCliExample(tool.key, deferredArgs, tool.params),
    [tool.key, tool.params, deferredArgs]
  );

  const curlSnippet = useMemo(() => {
    if (!showCurl) {
      return null;
    }
    if (!lastUrl) {
      return null;
    }
    return buildCurlExample(lastUrl, tool.mode === "write" ? "POST" : "GET");
  }, [showCurl, lastUrl, tool.mode]);

  const [activeTab, setActiveTab] = useState<string>("cli");
  const [open, setOpen] = useState(false);

  return (
    <div>
      <button
        className="flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-[0.65rem] text-muted-foreground transition-colors hover:text-foreground"
        onClick={() => setOpen((prev) => !prev)}
        type="button"
      >
        <Code className="size-3" />
        Code
        <ChevronDown
          className={cn(
            "ml-auto size-3 transition-transform duration-150",
            open && "rotate-180"
          )}
        />
      </button>

      {open && (
        <div className="mt-2">
          <div className="flex flex-col gap-2">
            <Tabs
              defaultValue={activeTab}
              onValueChange={(value) => {
                if (value !== null) {
                  setActiveTab(String(value));
                }
              }}
            >
              <div className="flex items-center justify-between">
                <TabsList>
                  <TabsTrigger value="cli">CLI</TabsTrigger>
                  {showCurl && <TabsTrigger value="curl">cURL</TabsTrigger>}
                </TabsList>
                <CopyButton
                  label="Copy code example"
                  text={activeTab === "curl" ? (curlSnippet ?? "") : cliSnippet}
                />
              </div>

              <TabsContent
                className="mt-2 overflow-auto rounded-md bg-muted/30 p-3"
                value="cli"
              >
                <pre className="whitespace-pre-wrap break-all font-mono text-foreground text-xs">
                  {cliSnippet}
                </pre>
              </TabsContent>

              {showCurl && (
                <TabsContent
                  className="mt-2 overflow-auto rounded-md bg-muted/30 p-3"
                  value="curl"
                >
                  {curlSnippet ? (
                    <pre className="whitespace-pre-wrap break-all font-mono text-foreground text-xs">
                      {curlSnippet}
                    </pre>
                  ) : (
                    <p className="text-muted-foreground text-xs italic">
                      Execute to see cURL
                    </p>
                  )}
                </TabsContent>
              )}
            </Tabs>
          </div>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// CopyCurlButton – single-click cURL copy for the response metadata bar
// ---------------------------------------------------------------------------

export function CopyCurlButton({
  url,
  method,
}: {
  url: string;
  method: string;
}) {
  const curlSnippet = buildCurlExample(url, method);
  const { copied, copy } = useCopy();

  return (
    <Button
      className="h-5 gap-1 px-1.5 text-[0.6rem]"
      onClick={() => copy(curlSnippet)}
      size="xs"
      variant="ghost"
    >
      {copied ? (
        <Check className="size-2.5 text-emerald-400" />
      ) : (
        <Copy className="size-2.5" />
      )}
      cURL
    </Button>
  );
}
