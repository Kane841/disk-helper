import type {
  AppSettings,
  AuditLogItem,
  CategoryStat,
  ChatMessage,
  CleanupSuggestion,
  FileNode,
  QuarantineItem,
  VolumeInfo,
} from "@/types";

export const mockVolume: VolumeInfo = {
  drive: "C:",
  total_bytes: 512_000_000_000,
  used_bytes: 420_000_000_000,
  free_bytes: 92_000_000_000,
  usage_percent: 82.0,
};

export const mockCategories: CategoryStat[] = [
  { code: "system", size_bytes: 68_000_000_000, ratio: 0.162 },
  { code: "program", size_bytes: 112_000_000_000, ratio: 0.267 },
  { code: "user_doc", size_bytes: 85_000_000_000, ratio: 0.202 },
  { code: "cache_temp", size_bytes: 52_000_000_000, ratio: 0.124 },
  { code: "download", size_bytes: 38_000_000_000, ratio: 0.09 },
  { code: "other", size_bytes: 65_000_000_000, ratio: 0.155 },
];

export const mockFileTree: FileNode[] = [
  {
    path: "C:\\",
    name: "C:\\",
    is_dir: true,
    size_bytes: 0,
    folder_size: 420_000_000_000,
    children: [
      {
        path: "C:\\Users\\AH",
        name: "AH",
        is_dir: true,
        size_bytes: 0,
        folder_size: 156_000_000_000,
        risk: "caution",
        children: [
          {
            path: "C:\\Users\\AH\\Downloads",
            name: "Downloads",
            is_dir: true,
            size_bytes: 0,
            folder_size: 38_000_000_000,
            risk: "caution",
          },
          {
            path: "C:\\Users\\AH\\AppData\\Local\\Temp",
            name: "Temp",
            is_dir: true,
            size_bytes: 0,
            folder_size: 12_500_000_000,
            risk: "safe",
          },
          {
            path: "C:\\Users\\AH\\Videos\\project4k.mp4",
            name: "project4k.mp4",
            is_dir: false,
            size_bytes: 8_400_000_000,
            folder_size: 8_400_000_000,
            extension: "mp4",
            modified_at: "2025-11-02T08:30:00Z",
            risk: "caution",
          },
        ],
      },
      {
        path: "C:\\Program Files",
        name: "Program Files",
        is_dir: true,
        size_bytes: 0,
        folder_size: 98_000_000_000,
        risk: "danger",
        coverage: "partial",
      },
      {
        path: "C:\\Windows",
        name: "Windows",
        is_dir: true,
        size_bytes: 0,
        folder_size: 68_000_000_000,
        risk: "danger",
        coverage: "partial",
        children: [
          {
            path: "C:\\Windows\\Temp",
            name: "Temp",
            is_dir: true,
            size_bytes: 0,
            folder_size: 4_200_000_000,
            risk: "safe",
          },
        ],
      },
      {
        path: "C:\\pagefile.sys",
        name: "pagefile.sys",
        is_dir: false,
        size_bytes: 16_000_000_000,
        folder_size: 16_000_000_000,
        risk: "danger",
      },
    ],
  },
];

export const mockTopFiles: FileNode[] = [
  {
    path: "C:\\Users\\AH\\Videos\\project4k.mp4",
    name: "project4k.mp4",
    is_dir: false,
    size_bytes: 8_400_000_000,
    folder_size: 8_400_000_000,
    modified_at: "2025-11-02T08:30:00Z",
    risk: "caution",
  },
  {
    path: "C:\\Users\\AH\\Downloads\\ubuntu-24.04.iso",
    name: "ubuntu-24.04.iso",
    is_dir: false,
    size_bytes: 6_200_000_000,
    folder_size: 6_200_000_000,
    modified_at: "2024-08-15T12:00:00Z",
    risk: "caution",
  },
  {
    path: "C:\\pagefile.sys",
    name: "pagefile.sys",
    is_dir: false,
    size_bytes: 16_000_000_000,
    folder_size: 16_000_000_000,
    risk: "danger",
  },
  {
    path: "C:\\Users\\AH\\AppData\\Local\\Docker\\wsl\\data.vhdx",
    name: "data.vhdx",
    is_dir: false,
    size_bytes: 24_000_000_000,
    folder_size: 24_000_000_000,
    modified_at: "2026-01-10T09:00:00Z",
    risk: "caution",
  },
  {
    path: "C:\\Users\\AH\\Documents\\backup.zip",
    name: "backup.zip",
    is_dir: false,
    size_bytes: 3_100_000_000,
    folder_size: 3_100_000_000,
    modified_at: "2025-06-01T18:20:00Z",
    risk: "safe",
  },
];

export const mockTopFolders: FileNode[] = [
  {
    path: "C:\\Program Files",
    name: "Program Files",
    is_dir: true,
    size_bytes: 0,
    folder_size: 98_000_000_000,
    risk: "danger",
  },
  {
    path: "C:\\Users\\AH",
    name: "AH",
    is_dir: true,
    size_bytes: 0,
    folder_size: 156_000_000_000,
    risk: "caution",
  },
  {
    path: "C:\\Windows",
    name: "Windows",
    is_dir: true,
    size_bytes: 0,
    folder_size: 68_000_000_000,
    risk: "danger",
  },
  {
    path: "C:\\Users\\AH\\AppData\\Local",
    name: "Local",
    is_dir: true,
    size_bytes: 0,
    folder_size: 42_000_000_000,
    risk: "caution",
  },
];

export const mockSuggestions: CleanupSuggestion[] = [
  {
    path: "C:\\Users\\AH\\AppData\\Local\\Temp",
    is_dir: true,
    size_bytes: 12_500_000_000,
    risk: "safe",
    category: "temp",
    rule_id: "user_temp",
    description: "用户临时文件目录，应用重启后可能重建",
    restore_hint: "可从隔离区还原",
    last_modified: "2026-06-20T10:00:00Z",
  },
  {
    path: "C:\\Windows\\Temp",
    is_dir: true,
    size_bytes: 4_200_000_000,
    risk: "safe",
    category: "temp",
    rule_id: "win_temp",
    description: "Windows 系统临时文件",
    restore_hint: "可从隔离区还原；系统会自动重建",
    last_modified: "2026-06-21T08:00:00Z",
  },
  {
    path: "C:\\Users\\AH\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache",
    is_dir: true,
    size_bytes: 2_800_000_000,
    risk: "safe",
    category: "browser_cache",
    rule_id: "chrome_cache",
    description: "Chrome 浏览器缓存",
    restore_hint: "清理后浏览器会重新下载页面资源",
    last_modified: "2026-06-22T06:00:00Z",
  },
  {
    path: "C:\\Users\\AH\\Downloads\\old-installer.exe",
    is_dir: false,
    size_bytes: 520_000_000,
    risk: "caution",
    category: "download_stale",
    rule_id: "download_stale_90d",
    description: "下载目录中超过 90 天未修改的文件",
    restore_hint: "可从隔离区还原",
    last_modified: "2025-01-12T14:00:00Z",
  },
  {
    path: "C:\\Users\\AH\\AppData\\Local\\npm-cache",
    is_dir: true,
    size_bytes: 1_900_000_000,
    risk: "caution",
    category: "app_cache",
    rule_id: "npm_cache",
    description: "npm 包缓存，删除后下次安装会重新下载",
    restore_hint: "可从隔离区还原",
    last_modified: "2026-05-01T11:00:00Z",
  },
  {
    path: "C:\\$Recycle.Bin",
    is_dir: true,
    size_bytes: 3_600_000_000,
    risk: "safe",
    category: "recycle",
    rule_id: "recycle_bin",
    description: "系统回收站内容",
    restore_hint: "清空前请确认回收站无需要的文件",
    last_modified: "2026-06-18T16:00:00Z",
  },
  {
    path: "C:\\Windows\\System32\\drivers",
    is_dir: true,
    size_bytes: 890_000_000,
    risk: "danger",
    category: "other",
    rule_id: "windows_dir",
    description: "系统驱动目录，误删可能导致设备无法工作",
    restore_hint: "强烈不建议清理；仅演示危险项保护",
    last_modified: "2026-03-01T00:00:00Z",
  },
];

export const mockQuarantineItems: QuarantineItem[] = [
  {
    id: "q-001",
    original_path: "C:\\Users\\AH\\AppData\\Local\\Temp\\cache_old.dat",
    quarantine_path: "C:\\Users\\AH\\AppData\\Roaming\\DiskHelper\\quarantine\\2026-06-20\\q-001",
    size_bytes: 256_000_000,
    moved_at: "2026-06-20T15:30:00Z",
    expires_at: "2026-07-20T15:30:00Z",
    status: "active",
    risk: "safe",
  },
  {
    id: "q-002",
    original_path: "C:\\Users\\AH\\Downloads\\setup_v1.zip",
    quarantine_path: "C:\\Users\\AH\\AppData\\Roaming\\DiskHelper\\quarantine\\2026-06-18\\q-002",
    size_bytes: 128_000_000,
    moved_at: "2026-06-18T10:00:00Z",
    expires_at: "2026-07-18T10:00:00Z",
    status: "expiring",
    risk: "caution",
  },
];

export const mockAuditLogs: AuditLogItem[] = [
  {
    id: "log-001",
    occurred_at: "2026-06-22T07:00:00Z",
    event_type: "scan_complete",
    summary: "C 盘全量扫描完成，共 284,521 个文件",
    result: "success",
  },
  {
    id: "log-002",
    occurred_at: "2026-06-20T15:30:00Z",
    event_type: "soft_delete",
    summary: "移入隔离区 1 项，释放 244 MB",
    result: "success",
    related_path: "C:\\Users\\AH\\AppData\\Local\\Temp\\cache_old.dat",
  },
  {
    id: "log-003",
    occurred_at: "2026-06-20T14:10:00Z",
    event_type: "ai_query",
    summary: "AI 咨询成功",
    result: "info",
  },
  {
    id: "log-004",
    occurred_at: "2026-06-18T10:00:00Z",
    event_type: "soft_delete",
    summary: "移入隔离区 1 项，释放 122 MB",
    result: "success",
    related_path: "C:\\Users\\AH\\Downloads\\setup_v1.zip",
  },
  {
    id: "log-005",
    occurred_at: "2026-06-18T09:55:00Z",
    event_type: "scan_start",
    summary: "开始 C 盘全量扫描",
    result: "info",
  },
  {
    id: "log-006",
    occurred_at: "2026-06-15T20:00:00Z",
    event_type: "settings_change",
    summary: "AI 模式切换为本地 Ollama",
    result: "info",
  },
  {
    id: "log-007",
    occurred_at: "2026-06-10T11:30:00Z",
    event_type: "restore",
    summary: "从隔离区还原 1 项",
    result: "success",
  },
  {
    id: "log-008",
    occurred_at: "2026-06-05T16:00:00Z",
    event_type: "purge",
    summary: "永久删除隔离区 2 项",
    result: "success",
  },
];

export const mockAiMessages: ChatMessage[] = [
  {
    id: "msg-1",
    role: "user",
    content: "AppData\\Local\\Temp 下 12GB 能删吗？",
    sent_at: "2026-06-22T08:00:00Z",
  },
  {
    id: "msg-2",
    role: "assistant",
    content:
      "**结论**：可以清理，风险等级为 **安全**。\n\n**是什么**：用户级临时目录，存放应用运行时产生的临时文件。\n\n**影响**：部分应用下次启动可能稍慢重建缓存；不会影响系统启动。\n\n**恢复**：已移入隔离区可在 30 天内还原。",
    sent_at: "2026-06-22T08:00:05Z",
  },
];

export const mockSettings: AppSettings = {
  theme: "system",
  ai_mode: "local",
  ollama_base_url: "http://127.0.0.1:11434",
  ollama_model: "deepseek-r1:1.5b",
  has_api_key: false,
  quarantine_root: "C:\\Users\\AH\\AppData\\Roaming\\DiskHelper\\quarantine",
  retention_days: 30,
  admin_scan_enabled: false,
  warning_threshold_gb: 10,
  critical_threshold_gb: 2,
  soft_delete_target: "quarantine",
};

export function findNodeByPath(
  nodes: FileNode[],
  targetPath: string,
): FileNode | undefined {
  for (const node of nodes) {
    if (node.path === targetPath) return node;
    if (node.children) {
      const found = findNodeByPath(node.children, targetPath);
      if (found) return found;
    }
  }
  return undefined;
}

export function getChildrenOf(path: string): FileNode[] {
  const root = mockFileTree[0];
  if (path === "C:\\") return root.children ?? [];
  const node = findNodeByPath(mockFileTree, path);
  return node?.children ?? [];
}

export function searchNodes(keyword: string): FileNode[] {
  const results: FileNode[] = [];
  const kw = keyword.toLowerCase();

  function walk(nodes: FileNode[]) {
    for (const n of nodes) {
      if (n.name.toLowerCase().includes(kw) || n.path.toLowerCase().includes(kw)) {
        results.push(n);
      }
      if (n.children) walk(n.children);
    }
  }
  walk(mockFileTree);
  return results.slice(0, 200);
}
