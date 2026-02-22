// Mirrors ParamType enum from crates/earl-core/src/schema.rs
export type ParamType =
  | "string"
  | "integer"
  | "number"
  | "boolean"
  | "null"
  | "array"
  | "object";

// Mirrors ParamSpec struct from crates/earl-core/src/schema.rs
export interface ParamSpec {
  default?: unknown;
  description?: string;
  name: string;
  required: boolean;
  type: ParamType;
}

// Mirrors ToolSourceResponse from src/web/mod.rs
export interface ToolSource {
  path: string;
  scope: "local" | "global";
}

// Mirrors ToolSummaryResponse from src/web/mod.rs
export interface Tool {
  categories: string[];
  command: string;
  description: string;
  example_cli: string;
  key: string;
  mode: "read" | "write";
  params: ParamSpec[];
  protocol: "http" | "graphql" | "grpc" | "bash" | "sql";
  provider: string;
  secrets: string[];
  source: ToolSource;
  summary: string;
  title: string;
}

// Mirrors ToolDetailResponse from src/web/mod.rs
export interface ToolDetail extends Tool {
  operation: unknown;
}

// Mirrors ExecuteRequest from src/web/mod.rs
export interface ExecuteRequest {
  args: Record<string, unknown>;
  command: string;
  confirm_write?: boolean;
}

// Mirrors ExecuteSuccessResponse from src/web/mod.rs
export interface ExecuteResponse {
  decoded: unknown;
  human_output: string;
  key: string;
  mode: string;
  result: unknown;
  status: number;
  url: string;
}

// Mirrors ValidateRequest from src/web/mod.rs
export interface ValidateRequest {
  args: Record<string, unknown>;
  command: string;
}

// Mirrors ValidateResponse from src/web/mod.rs
export interface ValidateResponse {
  bound_params?: string[];
  command_key?: string;
  error?: {
    code: string;
    message: string;
  };
  missing_required?: string[];
  valid: boolean;
}

// Mirrors SecretsStatusRequest from src/web/mod.rs
export interface SecretsStatusRequest {
  secrets: string[];
}

// Response from POST /api/secrets/status
export type SecretsStatus = Record<string, { configured: boolean }>;

// API error envelope from ApiError::into_response
export interface ApiError {
  error: {
    code: string;
    message: string;
  };
}

// Grouped tools for sidebar rendering
export interface ProviderGroup {
  provider: string;
  tools: Tool[];
}

// Execution state machine for playground
export type ExecutionState =
  | { status: "idle" }
  | {
      status: "loading";
      abortController: AbortController;
      previousResponse?: ExecuteResponse;
    }
  | { status: "success"; response: ExecuteResponse; timing: number }
  | { status: "error"; error: ApiError; timing?: number };

// History entry for request history
export interface HistoryEntry {
  args: Record<string, unknown>;
  command: string;
  error?: ApiError;
  id: string;
  response?: ExecuteResponse;
  responseSummary: string;
  status: number | null;
  timestamp: number;
}
