import type { ParamSpec } from "./types";

const SHELL_SPECIAL_RE = /[\s"'\\$`!#&|;()<>]/;

function shellQuote(s: string): string {
  if (s === "") {
    return '""';
  }
  if (SHELL_SPECIAL_RE.test(s)) {
    return `'${s.replace(/'/g, "'\\''")}'`;
  }
  return s;
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) {
    return "null";
  }
  if (typeof value === "string") {
    return shellQuote(value);
  }
  if (typeof value === "boolean" || typeof value === "number") {
    return String(value);
  }
  return shellQuote(JSON.stringify(value));
}

export function buildCliExample(
  commandKey: string,
  args: Record<string, unknown>,
  params: ParamSpec[]
): string {
  const parts = [`earl call ${commandKey}`];

  for (const param of params) {
    const value = args[param.name];
    if (value === undefined || value === null || value === "") {
      continue;
    }

    // Skip if value equals the default
    if (
      param.default !== undefined &&
      JSON.stringify(value) === JSON.stringify(param.default)
    ) {
      continue;
    }

    parts.push(`--${param.name} ${formatValue(value)}`);
  }

  return parts.join(" \\\n  ");
}

export function buildCurlExample(url: string, method = "GET"): string {
  const parts = ["curl"];
  if (method !== "GET") {
    parts.push(`-X ${method}`);
  }
  parts.push(shellQuote(url));
  return parts.join(" ");
}
