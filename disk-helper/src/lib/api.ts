import { mockApi } from "@/mocks/mock-api";
import { invokeApi } from "@/lib/tauri-client";
import type { FileNode, ScanStatus, SpaceCategory, VolumeInfo } from "@/types";

export const useMockApi = import.meta.env.VITE_USE_MOCK !== "false";

export interface ScanStatusPayload {
  status: ScanStatus;
  progress_percent: number;
  scanned_files: number;
  skipped_files: number;
  last_completed_at: string | null;
}

export interface CategoryStatPayload {
  code: SpaceCategory;
  size_bytes: number;
  ratio: number;
}

const tauriIpcApi = {
  volumeGetCDrive(): Promise<VolumeInfo> {
    return invokeApi<VolumeInfo>("volume_get_c_drive");
  },

  scanStart(type: "full" | "incremental"): Promise<{ scan_run_id: string }> {
    return invokeApi("scan_start", { scanType: type });
  },

  scanPause(): Promise<{ status: string }> {
    return invokeApi("scan_pause");
  },

  scanResume(): Promise<{ status: string }> {
    return invokeApi("scan_resume");
  },

  scanCancel(): Promise<{ status: string }> {
    return invokeApi("scan_cancel");
  },

  scanGetStatus(): Promise<ScanStatusPayload> {
    return invokeApi("scan_get_status");
  },

  async indexGetCategoryStats(): Promise<CategoryStatPayload[]> {
    const resp = await invokeApi<{ categories: CategoryStatPayload[] }>(
      "index_get_category_stats",
    );
    return resp.categories;
  },

  async indexGetChildren(path: string, sort: "size" | "name" = "size"): Promise<FileNode[]> {
    const resp = await invokeApi<{ nodes: FileNode[] }>("index_get_children", { path, sort });
    return resp.nodes;
  },

  async indexSearch(keyword: string, limit = 100): Promise<FileNode[]> {
    const resp = await invokeApi<{ items: FileNode[] }>("index_search", { keyword, limit });
    return resp.items;
  },

  async indexGetTopFiles(scopePath = "C:\\", limit = 100): Promise<FileNode[]> {
    const resp = await invokeApi<{ items: FileNode[] }>("index_get_top_files", {
      scopePath,
      limit,
    });
    return resp.items;
  },

  async indexGetTopFolders(scopePath = "C:\\", limit = 100): Promise<FileNode[]> {
    const resp = await invokeApi<{ items: FileNode[] }>("index_get_top_folders", {
      scopePath,
      limit,
    });
    return resp.items;
  },
};

/** Browser prototype uses mock; Tauri desktop uses real IPC when VITE_USE_MOCK=false. */
export const api = useMockApi
  ? mockApi
  : {
      ...mockApi,
      ...tauriIpcApi,
    };
