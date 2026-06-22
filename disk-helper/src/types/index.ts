export type RiskLevel = "safe" | "caution" | "danger";
export type ScanStatus = "idle" | "running" | "paused" | "completed" | "failed";
export type AiMode = "local" | "cloud";
export type SpaceCategory =
  | "system"
  | "program"
  | "user_doc"
  | "cache_temp"
  | "download"
  | "other";

export interface VolumeInfo {
  drive: string;
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  usage_percent: number;
}

export interface CategoryStat {
  code: SpaceCategory;
  size_bytes: number;
  ratio: number;
}

export interface FileNode {
  path: string;
  name: string;
  is_dir: boolean;
  size_bytes: number;
  folder_size: number;
  modified_at?: string;
  extension?: string;
  coverage?: "full" | "partial" | "skipped";
  risk?: RiskLevel;
  children?: FileNode[];
}

export interface CleanupSuggestion {
  path: string;
  is_dir: boolean;
  size_bytes: number;
  risk: RiskLevel;
  category: string;
  rule_id: string;
  description: string;
  restore_hint: string;
  last_modified?: string;
}

export interface QuarantineItem {
  id: string;
  original_path: string;
  quarantine_path: string;
  size_bytes: number;
  moved_at: string;
  expires_at: string;
  status: "active" | "expiring" | "expired";
  risk: RiskLevel;
}

export interface AuditLogItem {
  id: string;
  occurred_at: string;
  event_type: string;
  summary: string;
  result: "success" | "partial" | "failed" | "info";
  related_path?: string;
}

export interface ContextItem {
  path: string;
  size_bytes: number;
  is_dir: boolean;
  risk?: RiskLevel;
  rule_description?: string;
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  sent_at: string;
}

export interface AppSettings {
  theme: "system" | "light" | "dark";
  ai_mode: AiMode;
  ollama_base_url: string;
  ollama_model: string;
  has_api_key: boolean;
  quarantine_root: string;
  retention_days: number;
  admin_scan_enabled: boolean;
  warning_threshold_gb: number;
  critical_threshold_gb: number;
  soft_delete_target: "quarantine" | "recycle_bin";
}
