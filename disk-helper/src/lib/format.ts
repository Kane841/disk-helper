const UNITS = ["B", "KB", "MB", "GB", "TB"] as const;

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const i = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    UNITS.length - 1,
  );
  const value = bytes / 1024 ** i;
  return `${value >= 100 ? value.toFixed(0) : value.toFixed(1)} ${UNITS[i]}`;
}

export function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

export function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export const CATEGORY_LABELS: Record<string, string> = {
  system: "系统",
  program: "程序",
  user_doc: "用户文档",
  cache_temp: "缓存/临时",
  download: "下载",
  other: "其他",
};

export const RISK_LABELS: Record<string, string> = {
  safe: "安全",
  caution: "谨慎",
  danger: "危险",
};

export const EVENT_TYPE_LABELS: Record<string, string> = {
  scan_start: "扫描开始",
  scan_complete: "扫描完成",
  scan_fail: "扫描失败",
  soft_delete: "软删除",
  restore: "还原",
  purge: "永久删除",
  ai_query: "AI 咨询",
  settings_change: "设置变更",
};
