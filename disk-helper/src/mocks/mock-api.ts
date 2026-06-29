import type {
  AppSettings,
  AuditLogItem,
  CategoryStat,
  CleanupSuggestion,
  ContextItem,
  FileNode,
  QuarantineItem,
  RiskLevel,
  ScanStatus,
  VolumeInfo,
} from "@/types";
import {
  getChildrenOf,
  mockAuditLogs,
  mockCategories,
  mockQuarantineItems,
  mockSettings,
  mockSuggestions,
  mockTopFiles,
  mockTopFolders,
  mockVolume,
  searchNodes,
} from "./fixtures";

const delay = (ms = 200) => new Promise((r) => setTimeout(r, ms));

let settings = { ...mockSettings };
let quarantineItems = [...mockQuarantineItems];
let auditLogs = [...mockAuditLogs];
let scanStatus: ScanStatus = "completed";
let lastScanAt = "2026-06-22T07:00:00Z";

export const mockApi = {
  async volumeGetCDrive(): Promise<VolumeInfo> {
    await delay();
    return { ...mockVolume };
  },

  async indexGetCategoryStats(): Promise<CategoryStat[]> {
    await delay();
    return [...mockCategories];
  },

  async scanGetStatus(): Promise<{
    status: ScanStatus;
    progress_percent: number;
    scanned_files: number;
    skipped_files: number;
    last_completed_at: string | null;
  }> {
    await delay(50);
    return {
      status: scanStatus,
      progress_percent: scanStatus === "completed" ? 100 : 0,
      scanned_files: 284521,
      skipped_files: 142,
      last_completed_at: lastScanAt,
    };
  },

  async scanStart(_type: "full" | "incremental") {
    await delay(50);
    scanStatus = "running";
    return { scan_run_id: "mock-scan-run" };
  },

  async scanPause() {
    await delay(50);
    scanStatus = "paused";
    return { status: "paused" };
  },

  async scanResume() {
    await delay(50);
    scanStatus = "running";
    return { status: "running" };
  },

  async scanCancel() {
    await delay(50);
    scanStatus = "idle";
    return { status: "cancelled" };
  },

  async indexGetChildren(path: string): Promise<FileNode[]> {
    await delay();
    return getChildrenOf(path);
  },

  async indexSearch(keyword: string): Promise<FileNode[]> {
    await delay();
    return searchNodes(keyword);
  },

  async indexGetTopFiles(): Promise<FileNode[]> {
    await delay();
    return [...mockTopFiles];
  },

  async indexGetTopFolders(): Promise<FileNode[]> {
    await delay();
    return [...mockTopFolders];
  },

  async rulesGetSuggestions(filters?: {
    risk?: RiskLevel | "all";
    category?: string;
    path_keyword?: string;
    page?: number;
    size?: number;
  }): Promise<{ items: CleanupSuggestion[]; releasable_bytes: number; total: number }> {
    await delay();
    let items = [...mockSuggestions];
    if (filters?.risk && filters.risk !== "all") {
      items = items.filter((i) => i.risk === filters.risk);
    }
    if (filters?.category && filters.category !== "all") {
      items = items.filter((i) => i.category === filters.category);
    }
    if (filters?.path_keyword) {
      const kw = filters.path_keyword.toLowerCase();
      items = items.filter((i) => i.path.toLowerCase().includes(kw));
    }
    const releasable_bytes = items
      .filter((i) => i.risk === "safe")
      .reduce((s, i) => s + i.size_bytes, 0);
    const total = items.length;
    const page = Math.max(1, filters?.page ?? 1);
    const size = Math.max(1, filters?.size ?? 50);
    const start = (page - 1) * size;
    items = items.slice(start, start + size);
    return { items, releasable_bytes, total };
  },

  async quarantineList(): Promise<QuarantineItem[]> {
    await delay();
    return [...quarantineItems];
  },

  async quarantineRestore(ids: string[]): Promise<{
    restored: number;
    failed?: { id: string; reason: string }[];
  }> {
    await delay(400);
    quarantineItems = quarantineItems.filter((i) => !ids.includes(i.id));
    return { restored: ids.length };
  },

  async quarantinePurge(ids: string[], confirmText: string): Promise<{ purged_count: number }> {
    if (confirmText !== "永久删除") {
      throw new Error("请输入「永久删除」以继续");
    }
    await delay(400);
    quarantineItems = quarantineItems.filter((i) => !ids.includes(i.id));
    return { purged_count: ids.length };
  },

  async auditList(): Promise<AuditLogItem[]> {
    await delay();
    return [...auditLogs];
  },

  async auditExport(_format: "json" | "txt" = "json"): Promise<{ file_path: string }> {
    await delay(300);
    return { file_path: "mock-audit-export.txt" };
  },

  async auditClear(_confirmed = true): Promise<{ deleted: number }> {
    await delay(200);
    const count = auditLogs.length;
    auditLogs = [];
    return { deleted: count };
  },

  async cleanupExecute(params: {
    items: { path: string }[];
    target?: "quarantine" | "recycle_bin";
    dangerConfirmToken?: string;
  }): Promise<{ success_count: number; failed: { path: string; reason: string }[] }> {
    await delay(400);
    const paths = params.items.map((item) => item.path);
    for (const path of paths) {
      const sug = mockSuggestions.find((s) => s.path === path);
      if (!sug) continue;
      quarantineItems.unshift({
        id: `q-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        original_path: path,
        quarantine_path: `${settings.quarantine_root}\\${new Date().toISOString().slice(0, 10)}\\...`,
        size_bytes: sug.size_bytes,
        moved_at: new Date().toISOString(),
        expires_at: new Date(Date.now() + settings.retention_days * 86400000).toISOString(),
        status: "active",
        risk: sug.risk,
      });
      auditLogs.unshift({
        id: `log-${Date.now()}`,
        occurred_at: new Date().toISOString(),
        event_type: "soft_delete",
        summary: `移入隔离区：${path}`,
        result: "success",
        related_path: path,
      });
    }
    return { success_count: paths.length, failed: [] };
  },

  async configGet(): Promise<AppSettings> {
    await delay();
    return { ...settings };
  },

  async indexClear(_confirmed = true): Promise<{ deleted_entries: number }> {
    await delay(300);
    return { deleted_entries: 0 };
  },

  async configSave(partial: Partial<AppSettings>): Promise<void> {
    await delay(300);
    settings = { ...settings, ...partial };
    if (partial.ai_mode !== undefined) {
      auditLogs = [
        {
          id: `log-${Date.now()}`,
          occurred_at: new Date().toISOString(),
          event_type: "settings_change",
          summary: `AI 模式切换为 ${partial.ai_mode === "local" ? "本地 Ollama" : "云端 DeepSeek"}`,
          result: "info",
        },
        ...auditLogs,
      ];
    }
  },

  async aiChat(
    question: string,
    _context: ContextItem[],
  ): Promise<{ message: string; disclaimer: string }> {
    await delay(1000);
    const mode = settings.ai_mode === "local" ? "Ollama (deepseek-r1:1.5b)" : "DeepSeek API";
    return {
      message: `【${mode} 模拟回复】\n\n关于「${question}」：根据规则引擎，所选路径多为**安全/谨慎**级别。临时目录与浏览器缓存通常可清理；系统目录请勿删除。\n\n如需执行清理，请在「安全清理」页勾选确认后移入隔离区。`,
      disclaimer: "以上仅供参考，删除操作请在安全清理中自行确认。",
    };
  },

  async aiTestConnection(): Promise<{ status: string; message: string }> {
    await delay(800);
    if (settings.ai_mode === "local") {
      return { status: "success", message: "Ollama 连接成功（模拟）" };
    }
    if (!settings.has_api_key) {
      return { status: "failed", message: "请先配置 API Key" };
    }
    return { status: "success", message: "DeepSeek API 连接成功（模拟）" };
  },

  setScanStatus(status: ScanStatus) {
    scanStatus = status;
    if (status === "completed") {
      lastScanAt = new Date().toISOString();
    }
  },

  addQuarantineFromCleanup(paths: string[]) {
    for (const path of paths) {
      const sug = mockSuggestions.find((s) => s.path === path);
      if (!sug) continue;
      quarantineItems.unshift({
        id: `q-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        original_path: path,
        quarantine_path: `${settings.quarantine_root}\\${new Date().toISOString().slice(0, 10)}\\...`,
        size_bytes: sug.size_bytes,
        moved_at: new Date().toISOString(),
        expires_at: new Date(Date.now() + settings.retention_days * 86400000).toISOString(),
        status: "active",
        risk: sug.risk,
      });
      auditLogs.unshift({
        id: `log-${Date.now()}`,
        occurred_at: new Date().toISOString(),
        event_type: "soft_delete",
        summary: `移入隔离区：${path}`,
        result: "success",
        related_path: path,
      });
    }
  },
};

export type MockApi = typeof mockApi;
