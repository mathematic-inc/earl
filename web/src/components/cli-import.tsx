"use client";

import { Terminal } from "lucide-react";
import { useCallback, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface CliImportProps {
  onImport: (command: string, args: Record<string, string>) => void;
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

interface ParseResult {
  args: Record<string, string>;
  command: string;
}

interface TokeniserState {
  current: string;
  inDouble: boolean;
  inSingle: boolean;
  isEscaped: boolean;
  tokens: string[];
}

/** Process a single character and return whether it was consumed as a control char. */
function processChar(ch: string, state: TokeniserState): void {
  if (state.isEscaped) {
    state.current += ch;
    state.isEscaped = false;
    return;
  }
  if (ch === "\\") {
    state.isEscaped = true;
    return;
  }
  if (ch === "'" && !state.inDouble) {
    state.inSingle = !state.inSingle;
    return;
  }
  if (ch === '"' && !state.inSingle) {
    state.inDouble = !state.inDouble;
    return;
  }
  if ((ch === " " || ch === "\t") && !state.inSingle && !state.inDouble) {
    if (state.current.length > 0) {
      state.tokens.push(state.current);
      state.current = "";
    }
    return;
  }
  state.current += ch;
}

/**
 * Tokenise a CLI string into an array of arguments, handling quoted values and
 * escaped characters. Supports both single and double quotes as well as
 * backslash escapes.
 */
function tokenise(input: string): string[] {
  const state: TokeniserState = {
    tokens: [],
    current: "",
    inSingle: false,
    inDouble: false,
    isEscaped: false,
  };

  for (const ch of input) {
    processChar(ch, state);
  }

  if (state.isEscaped) {
    throw new Error("Trailing backslash");
  }
  if (state.inSingle || state.inDouble) {
    throw new Error("Unterminated quote");
  }
  if (state.current.length > 0) {
    state.tokens.push(state.current);
  }

  return state.tokens;
}

/** Strip the optional `earl call` prefix from tokens. */
function stripPrefix(tokens: string[]): string[] {
  let startIndex = 0;
  if (tokens[0] === "earl") {
    startIndex = 1;
    if (tokens[1] === "call") {
      startIndex = 2;
    }
  }
  return tokens.slice(startIndex);
}

/** Parse `--key value` pairs from the remaining tokens starting at index `i`. */
function parseFlags(
  remaining: string[],
  startIndex: number
): Record<string, string> {
  const args: Record<string, string> = {};
  let i = startIndex;

  while (i < remaining.length) {
    const token = remaining[i] ?? "";

    if (!token.startsWith("--")) {
      throw new Error(`Unexpected argument: ${token}`);
    }

    const key = token.slice(2);
    if (key.length === 0) {
      throw new Error("Empty flag name");
    }

    const eqIndex = key.indexOf("=");
    if (eqIndex !== -1) {
      args[key.slice(0, eqIndex)] = key.slice(eqIndex + 1);
      i++;
      continue;
    }

    const nextToken = remaining[i + 1];
    if (nextToken === undefined || nextToken.startsWith("--")) {
      args[key] = "true";
      i++;
      continue;
    }

    args[key] = nextToken;
    i += 2;
  }

  return args;
}

/**
 * Parse a CLI command string of the form:
 *
 *   earl call provider.command --key value --key2 value2
 *
 * Returns the command key (first positional argument after stripping the
 * `earl call` prefix) and all `--key value` pairs.
 */
function parseCliCommand(input: string): ParseResult {
  const trimmed = input.trim();
  if (trimmed.length === 0) {
    throw new Error("Empty input");
  }

  const tokens = tokenise(trimmed);
  const remaining = stripPrefix(tokens);
  if (remaining.length === 0) {
    throw new Error("No command specified");
  }

  // Find the command (first non-flag argument)
  let command: string | undefined;
  let i = 0;
  while (i < remaining.length) {
    const token = remaining[i] ?? "";
    if (token.startsWith("--")) {
      break;
    }
    if (command === undefined) {
      command = token;
    }
    i++;
  }

  if (command === undefined) {
    throw new Error("No command specified");
  }

  const args = parseFlags(remaining, i);
  return { command, args };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function CliImport({ onImport }: CliImportProps) {
  const [open, setOpen] = useState(false);
  const [value, setValue] = useState("");
  const [error, setError] = useState<string | undefined>(undefined);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      setValue(e.target.value);
      setError(undefined);
    },
    []
  );

  const handleSubmit = useCallback(() => {
    try {
      const result = parseCliCommand(value);
      onImport(result.command, result.args);
      setOpen(false);
      setValue("");
      setError(undefined);
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError(`Could not parse CLI command: ${err.message}`);
      } else {
        setError("Could not parse CLI command");
      }
    }
  }, [value, onImport]);

  const handleOpenChange = useCallback((nextOpen: boolean) => {
    setOpen(nextOpen);
    if (!nextOpen) {
      setValue("");
      setError(undefined);
    }
  }, []);

  return (
    <Dialog onOpenChange={handleOpenChange} open={open}>
      <DialogTrigger render={<Button size="sm" variant="ghost" />}>
        <Terminal className="size-3" />
        Import from CLI
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle>Import from CLI</DialogTitle>
        </DialogHeader>

        <div className="space-y-3">
          <p className="text-muted-foreground text-xs">
            Paste an{" "}
            <code className="rounded bg-muted px-1 py-0.5 font-mono text-[0.65rem]">
              earl call
            </code>{" "}
            command to auto-fill the form.
          </p>

          <Textarea
            aria-invalid={error !== undefined ? true : undefined}
            className="font-mono text-xs"
            onChange={handleChange}
            placeholder="earl call provider.command --key value --key2 value2"
            rows={3}
            value={value}
          />

          {error !== undefined && (
            <p className="text-destructive text-xs" role="alert">
              {error}
            </p>
          )}
        </div>

        <DialogFooter>
          <Button disabled={value.trim().length === 0} onClick={handleSubmit}>
            Import
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
