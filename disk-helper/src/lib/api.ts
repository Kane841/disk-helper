import { mockApi } from "@/mocks/mock-api";
import { invokeApi } from "@/lib/tauri-client";
import type {
  AppSettings,
  AuditLogItem,
  CleanupSuggestion,
  ContextItem,
  FileNode,
  QuarantineItem,
  ScanStatus,
  SpaceCategory,
  VolumeInfo,
} from "@/types";

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

  async rulesGetSuggestions(filters?: {
    risk?: string;
    category?: string;
    path_keyword?: string;
    page?: number;
    size?: number;
  }): Promise<{ items: CleanupSuggestion[]; releasable_bytes: number; total?: number }> {
    return invokeApi("rules_get_suggestions", {
      riskFilter: filters?.risk,
      categoryFilter: filters?.category,
      pathKeyword: filters?.path_keyword,
      page: filters?.page,
      size: filters?.size,
    });
  },

  async cleanupExecute(params: {
    items: { path: string }[];
    target?: "quarantine" | "recycle_bin";
    dangerConfirmToken?: string;
  }): Promise<{ success_count: number; failed: { path: string; reason: string }[] }> {
    return invokeApi("cleanup_execute", {
      items: params.items,
      target: params.target ?? "quarantine",
      dangerConfirmToken: params.dangerConfirmToken,
    });
  },

  async quarantineList(keyword?: string): Promise<QuarantineItem[]> {
    const resp = await invokeApi<{ items: QuarantineItem[] }>("quarantine_list", {
      keyword,
    });
    return resp.items;
  },

  async quarantineRestore(
    ids: string[],
    conflictStrategy?: "overwrite" | "alternate",
  ): Promise<{ restored: number; failed?: { id: string; reason: string }[] }> {
    return invokeApi("quarantine_restore", { ids, conflictStrategy });
  },

  async quarantinePurge(ids: string[], confirmText: string): Promise<{ purged_count: number }> {
    return invokeApi("quarantine_purge", { ids, confirmText });
  },

  async auditList(params?: {
    eventType?: string;
    keyword?: string;
    page?: number;
    size?: number;
  }): Promise<AuditLogItem[]> {
    const resp = await invokeApi<{ items: AuditLogItem[] }>("audit_list", {
      eventType: params?.eventType,
      keyword: params?.keyword,
      page: params?.page,
      size: params?.size,
    });
    return resp.items;
  },

  async auditExport(format: "json" | "txt" = "json", limit = 200): Promise<{ file_path: string }> {
    return invokeApi("audit_export", { format, limit });
  },

  async auditClear(confirmed = true): Promise<{ deleted: number }> {
    return invokeApi("audit_clear", { confirmed });
  },

  configGet(): Promise<AppSettings> {
    return invokeApi("config_get");
  },

  configSave(partial: Partial<AppSettings> & { api_key?: string }): Promise<AppSettings> {
    return invokeApi("config_save", {
      theme: partial.theme,
      aiMode: partial.ai_mode,
      ollamaBaseUrl: partial.ollama_base_url,
      ollamaModel: partial.ollama_model,
      apiKey: partial.api_key,
      quarantineRoot: partial.quarantine_root,
      retentionDays: partial.retention_days,
      adminScanEnabled: partial.admin_scan_enabled,
      warningThresholdGb: partial.warning_threshold_gb,
      criticalThresholdGb: partial.critical_threshold_gb,
      softDeleteTarget: partial.soft_delete_target,
    });
  },

  aiChat(
    question: string,
    contextItems: ContextItem[],
  ): Promise<{ message: string; disclaimer: string }> {
    return invokeApi("ai_chat", { question, contextItems });
  },

  aiTestConnection(params?: {
    ai_mode?: AppSettings["ai_mode"];
    api_key?: string;
  }): Promise<{ status: string; message: string; provider: string }> {
    return invokeApi("ai_test_connection", {
      aiMode: params?.ai_mode,
      apiKey: params?.api_key,
    });
  },
};

/** Browser prototype uses mock; Tauri desktop uses real IPC when VITE_USE_MOCK=false. */
export const api = useMockApi
  ? mockApi
  : {
      ...mockApi,
      ...tauriIpcApi,
    };

