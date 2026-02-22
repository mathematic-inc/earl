import { useEffect, useRef, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { checkSecrets } from "@/lib/api";
import type { ParamSpec, SecretsStatus, Tool } from "@/lib/types";
import { cn } from "@/lib/utils";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface CommandDocsProps {
  loading: boolean;
  onRefresh: () => void;
  tool: Tool | null;
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

/** Monochrome protocol badge */
function ProtocolBadge({ protocol }: { protocol: Tool["protocol"] }) {
  return (
    <Badge className="font-mono uppercase tracking-wider" variant="outline">
      {protocol}
    </Badge>
  );
}

/** Green (read) / red (write) mode badge */
function ModeBadge({ mode }: { mode: Tool["mode"] }) {
  return (
    <Badge
      className={cn(
        "uppercase tracking-wider",
        mode === "read"
          ? "border-emerald-300 bg-emerald-50 text-emerald-700"
          : "border-red-300 bg-red-50 text-red-700"
      )}
    >
      {mode}
    </Badge>
  );
}

/** Category badge */
function CategoryBadge({ category }: { category: string }) {
  return <Badge variant="secondary">{category}</Badge>;
}

// ---------------------------------------------------------------------------
// Secrets status
// ---------------------------------------------------------------------------

function SecretsSection({
  secrets,
  status,
}: {
  secrets: string[];
  status: SecretsStatus | null;
}) {
  if (secrets.length === 0) {
    return null;
  }

  return (
    <section className="space-y-2">
      <h2 className="font-semibold text-foreground text-sm tracking-tight">
        Secrets
      </h2>
      <ul className="space-y-1.5">
        {secrets.map((name) => {
          const configured = status?.[name]?.configured ?? false;
          return (
            <li className="flex items-center gap-2 text-sm" key={name}>
              <span
                className={cn(
                  "inline-block size-2 shrink-0 rounded-full",
                  configured ? "bg-emerald-500" : "bg-red-500"
                )}
              />
              <code className="font-mono text-xs">{name}</code>
              {!configured && (
                <span className="text-muted-foreground">
                  &mdash; not configured. Run{" "}
                  <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs">
                    earl secrets set {name}
                  </code>
                </span>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Parameters section
// ---------------------------------------------------------------------------

function ParamRow({ param, index }: { param: ParamSpec; index: number }) {
  const delay = Math.min(index, 4) * 30;

  return (
    <div
      className={cn(
        "fade-in slide-in-from-bottom-1 flex animate-in flex-col gap-1 rounded-md px-3 py-2.5",
        index % 2 === 0 ? "bg-muted/40" : "bg-transparent"
      )}
      style={{ animationDelay: `${delay}ms`, animationFillMode: "both" }}
    >
      <div className="flex items-center gap-2">
        <code className="font-medium font-mono text-foreground text-sm">
          {param.name}
        </code>
        <Badge className="font-mono text-[0.6rem]" variant="outline">
          {param.type}
        </Badge>
        {param.required && (
          <span className="font-semibold text-[0.6rem] text-red-600 uppercase tracking-wider">
            required
          </span>
        )}
        {param.default !== undefined && (
          <span className="text-muted-foreground text-xs">
            default:{" "}
            <code className="font-mono">{JSON.stringify(param.default)}</code>
          </span>
        )}
      </div>
      {param.description && (
        <p className="text-muted-foreground text-sm leading-relaxed">
          {param.description}
        </p>
      )}
    </div>
  );
}

function ParametersSection({ params }: { params: ParamSpec[] }) {
  if (params.length === 0) {
    return null;
  }

  // Sort required params to the top, preserving relative order otherwise
  const sorted = [...params].sort((a, b) => {
    if (a.required && !b.required) {
      return -1;
    }
    if (!a.required && b.required) {
      return 1;
    }
    return 0;
  });

  return (
    <section className="space-y-3">
      <h2 className="font-semibold text-foreground text-sm tracking-tight">
        Parameters
      </h2>
      <div className="space-y-0.5">
        {sorted.map((p, i) => (
          <ParamRow index={i} key={p.name} param={p} />
        ))}
      </div>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Source section
// ---------------------------------------------------------------------------

function SourceSection({ source }: { source: Tool["source"] }) {
  return (
    <section className="space-y-2">
      <h2 className="font-semibold text-foreground text-sm tracking-tight">
        Source
      </h2>
      <div className="flex items-center gap-2 text-muted-foreground text-sm">
        <Badge className="capitalize" variant="outline">
          {source.scope}
        </Badge>
        <code className="break-all font-mono text-xs">{source.path}</code>
      </div>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Loading skeleton
// ---------------------------------------------------------------------------

function LoadingSkeleton() {
  return (
    <div className="space-y-6 p-10">
      {/* Title */}
      <Skeleton className="h-7 w-64" />
      {/* Subtitle */}
      <Skeleton className="h-4 w-40" />
      {/* Description lines */}
      <div className="space-y-2 pt-4">
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-4 w-5/6" />
      </div>
      {/* Parameter rows */}
      <div className="space-y-3 pt-6">
        <Skeleton className="h-5 w-28" />
        <Skeleton className="h-12 w-full rounded-md" />
        <Skeleton className="h-12 w-full rounded-md" />
        <Skeleton className="h-12 w-full rounded-md" />
        <Skeleton className="h-12 w-full rounded-md" />
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Empty state
// ---------------------------------------------------------------------------

function EmptyState() {
  return (
    <div className="flex h-full items-center justify-center p-10">
      <div className="max-w-md space-y-4 text-center">
        <p className="text-muted-foreground text-sm">
          Earl loads commands from your template files.
        </p>
        <pre className="rounded-md bg-muted px-4 py-3 text-left font-mono text-sm">
          earl templates install &lt;source&gt;
        </pre>
        <p className="text-muted-foreground text-sm">
          Or create a{" "}
          <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs">
            .hcl
          </code>{" "}
          file in{" "}
          <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs">
            ./templates/
          </code>{" "}
          to get started.
        </p>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Remark plugins (stable reference to avoid re-renders)
// ---------------------------------------------------------------------------

const remarkPlugins = [remarkGfm];

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function CommandDocs({
  tool,
  loading,
  onRefresh: _onRefresh,
}: CommandDocsProps) {
  // Cross-fade state
  const [visible, setVisible] = useState(true);
  const [displayedTool, setDisplayedTool] = useState<Tool | null>(tool);
  const scrollRef = useRef<HTMLDivElement>(null);
  const prevToolKey = useRef<string | null>(null);

  // Secrets status
  const [secretsStatus, setSecretsStatus] = useState<SecretsStatus | null>(
    null
  );

  // Cross-fade on command switch
  useEffect(() => {
    const newKey = tool?.key ?? null;

    if (newKey === prevToolKey.current) {
      // Same tool or both null — no transition, just update in place
      setDisplayedTool(tool);
      return;
    }

    // Fade out
    setVisible(false);

    const timer = setTimeout(() => {
      setDisplayedTool(tool);
      prevToolKey.current = newKey;
      // Scroll to top
      if (scrollRef.current) {
        scrollRef.current.scrollTop = 0;
      }
      // Fade in
      setVisible(true);
    }, 100);

    return () => {
      clearTimeout(timer);
    };
  }, [tool]);

  // Fetch secrets status when displayed tool changes
  useEffect(() => {
    if (!displayedTool || displayedTool.secrets.length === 0) {
      setSecretsStatus(null);
      return;
    }

    let cancelled = false;

    checkSecrets({ secrets: displayedTool.secrets })
      .then((status) => {
        if (!cancelled) {
          setSecretsStatus(status);
        }
      })
      .catch(() => {
        // Silently fail — secrets panel just won't show status
        if (!cancelled) {
          setSecretsStatus(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [displayedTool]);

  // Loading state
  if (loading) {
    return (
      <ScrollArea className="h-full">
        <LoadingSkeleton />
      </ScrollArea>
    );
  }

  // Empty state
  if (!displayedTool) {
    return <EmptyState />;
  }

  return (
    <ScrollArea className="h-full" ref={scrollRef}>
      <div
        className={cn(
          "p-10 transition-opacity",
          visible ? "opacity-100 duration-150" : "opacity-0 duration-100"
        )}
      >
        {/* Header */}
        <header className="space-y-3">
          <h1 className="font-semibold text-2xl text-foreground leading-tight">
            {displayedTool.title}
          </h1>
          <code className="block font-mono text-muted-foreground text-sm">
            {displayedTool.provider}.{displayedTool.command}
          </code>

          {/* Metadata row */}
          <div className="flex flex-wrap items-center gap-2 pt-1">
            <ProtocolBadge protocol={displayedTool.protocol} />

            <ModeBadge mode={displayedTool.mode} />

            {displayedTool.categories.map((cat) => (
              <CategoryBadge category={cat} key={cat} />
            ))}
          </div>
        </header>

        <Separator className="my-8" />

        {/* Description */}
        {displayedTool.description && (
          <section className="prose prose-sm max-w-none prose-code:rounded prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:font-mono prose-a:text-primary prose-code:text-foreground prose-headings:text-foreground prose-p:text-muted-foreground text-foreground prose-code:before:content-none prose-code:after:content-none">
            <Markdown remarkPlugins={remarkPlugins}>
              {displayedTool.description}
            </Markdown>
          </section>
        )}

        <div className="mt-8 space-y-8">
          {/* Secrets */}
          <SecretsSection
            secrets={displayedTool.secrets}
            status={secretsStatus}
          />

          {/* Parameters */}
          <ParametersSection params={displayedTool.params} />

          {/* Source */}
          <SourceSection source={displayedTool.source} />
        </div>
      </div>
    </ScrollArea>
  );
}
