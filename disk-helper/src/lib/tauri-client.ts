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

/** Invoke a Tauri command and unwrap the `{ data, error }` envelope. */
export async function invokeApi<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const response = await invoke<ApiResponse<T>>(command, args ?? {});
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
