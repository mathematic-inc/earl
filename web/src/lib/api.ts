import type {
  ApiError,
  ExecuteRequest,
  ExecuteResponse,
  SecretsStatus,
  SecretsStatusRequest,
  Tool,
  ToolDetail,
  ValidateRequest,
  ValidateResponse,
} from "./types";

function getToken(): string | null {
  return (
    document
      .querySelector('meta[name="earl-token"]')
      ?.getAttribute("content") ?? null
  );
}

class ApiClientError extends Error {
  readonly code: string;
  readonly status: number;

  constructor(code: string, message: string, status: number) {
    super(message);
    this.name = "ApiClientError";
    this.code = code;
    this.status = status;
  }
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    ...Object.fromEntries(
      Object.entries(options.headers ?? {}).filter(
        (e): e is [string, string] => typeof e[1] === "string"
      )
    ),
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  const response = await fetch(path, {
    ...options,
    headers,
  });

  if (!response.ok) {
    let apiError: ApiError;
    try {
      apiError = await response.json();
    } catch {
      throw new ApiClientError(
        "network_error",
        `HTTP ${response.status}: ${response.statusText}`,
        response.status
      );
    }
    throw new ApiClientError(
      apiError.error.code,
      apiError.error.message,
      response.status
    );
  }

  return response.json();
}

function post<T>(
  path: string,
  body: unknown,
  signal?: AbortSignal
): Promise<T> {
  return request<T>(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
    signal,
  });
}

export function fetchTools(): Promise<Tool[]> {
  return request<Tool[]>("/api/tools");
}

export function fetchToolDetail(key: string): Promise<ToolDetail> {
  return request<ToolDetail>(`/api/tools/${encodeURIComponent(key)}`);
}

export function executeCommand(
  req: ExecuteRequest,
  signal?: AbortSignal
): Promise<ExecuteResponse> {
  return post<ExecuteResponse>("/api/execute", req, signal);
}

export function validateParams(
  req: ValidateRequest,
  signal?: AbortSignal
): Promise<ValidateResponse> {
  return post<ValidateResponse>("/api/validate", req, signal);
}

export function checkSecrets(
  req: SecretsStatusRequest
): Promise<SecretsStatus> {
  return post<SecretsStatus>("/api/secrets/status", req);
}

export { ApiClientError };
