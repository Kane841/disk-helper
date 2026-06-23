import type { VolumeInfo } from "@/types";
import { formatBytes, formatPercent } from "@/lib/format";
import { cn } from "@/lib/cn";
import { text } from "@/lib/theme";
import { Card, CardBody } from "@/components/ui/card";

export function CapacityCard({ volume }: { volume: VolumeInfo }) {
  const isWarning = volume.free_bytes < 10 * 1024 ** 3;
  const isCritical = volume.free_bytes < 2 * 1024 ** 3;

  return (
    <Card strong>
      <CardBody>
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className={cn("text-xs font-semibold uppercase tracking-wider", text.muted)}>
              {volume.drive} 盘
            </p>
            <p className={cn("mt-2 text-3xl font-semibold tracking-tight", text.primary)}>
              {formatBytes(volume.free_bytes)}
              <span className={cn("ml-2 text-base font-normal", text.muted)}>可用</span>
            </p>
            <p className={cn("mt-2 text-sm", text.muted)}>
              已用 {formatBytes(volume.used_bytes)} / 共 {formatBytes(volume.total_bytes)}
            </p>
          </div>
          <div className="rounded-2xl bg-white/40 px-4 py-3 text-right ring-1 ring-white/50 backdrop-blur-sm dark:bg-white/5 dark:ring-white/10">
            <p className="text-2xl font-semibold text-emerald-600 dark:text-emerald-300">
              {formatPercent(volume.usage_percent)}
            </p>
            <p className={cn("text-xs", text.muted)}>使用率</p>
          </div>
        </div>
        <div className="mt-5 h-2.5 overflow-hidden rounded-full bg-white/40 ring-1 ring-white/30 dark:bg-white/5 dark:ring-white/10">
          <div
            className={`h-full rounded-full transition-all ${
              isCritical
                ? "bg-gradient-to-r from-red-500 to-red-400"
                : isWarning
                  ? "bg-gradient-to-r from-amber-500 to-amber-400"
                  : "bg-gradient-to-r from-emerald-600 to-emerald-400"
            }`}
            style={{ width: `${Math.min(volume.usage_percent, 100)}%` }}
          />
        </div>
        {(isWarning || isCritical) && (
          <p
            className={`mt-3 rounded-xl px-3 py-2 text-sm backdrop-blur-sm ${
              isCritical
                ? "bg-red-500/10 text-red-700 ring-1 ring-red-500/20 dark:text-red-300"
                : "bg-amber-500/10 text-amber-800 ring-1 ring-amber-500/20 dark:text-amber-300"
            }`}
          >
            {isCritical ? "空间严重不足" : "空间偏紧"} — 建议查看清理建议或扫描大文件
          </p>
        )}
      </CardBody>
    </Card>
  );
}
