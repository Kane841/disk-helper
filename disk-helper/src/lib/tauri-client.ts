import { invoke } from "@tauri-apps/api/core";

export interface ApiError {
  code: string;
  message: string;
  target?: string;
}

export interface ApiResponse<T> {
  data?: T;
  error?: ApiError;
}

export class TauriApiError extends Error {
  code: string;
  target?: string;

  constructor(error: ApiError) {
    super(error.message);
    this.name = "TauriApiError";
    this.code = error.code;
    this.target = error.target;
  }
}

function ipcErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return "IPC 调用失败";
}

/** Invoke a Tauri command and unwrap the `{ data, error }` envelope. */
export async function invokeApi<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  let response: ApiResponse<T>;
  try {
    response = await invoke<ApiResponse<T>>(command, args ?? {});
  } catch (error) {
    throw new TauriApiError({ code: "InternalError", message: ipcErrorMessage(error) });
  }
  if (response.error) {
    throw new TauriApiError(response.error);
  }
  if (response.data === undefined) {
    throw new TauriApiError({
      code: "InternalError",
      message: `Empty response from command: ${command}`,
    });
  }
  return response.data;
}
