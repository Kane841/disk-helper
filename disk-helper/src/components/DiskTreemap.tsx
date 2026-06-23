import type { FileNode } from "@/types";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/cn";
import { text } from "@/lib/theme";

interface DiskTreemapProps {
  items: FileNode[];
  selectedPath: string;
  onSelect: (node: FileNode) => void;
}

export function DiskTreemap({ items, selectedPath, onSelect }: DiskTreemapProps) {
  const total = items.reduce(
    (s, i) => s + (i.is_dir ? i.folder_size : i.size_bytes),
    0,
  );

  if (items.length === 0) {
    return (
      <div className={cn("flex h-full items-center justify-center text-sm", text.muted)}>
        无子项
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-[280px] flex-wrap gap-2 p-3">
      {items.map((item) => {
        const size = item.is_dir ? item.folder_size : item.size_bytes;
        const pct = total > 0 ? (size / total) * 100 : 0;
        const minWidth = Math.max(pct, 8);
        const selected = item.path === selectedPath;

        return (
          <button
            key={item.path}
            type="button"
            onClick={() => onSelect(item)}
            className={cn(
              "flex min-h-[76px] flex-col justify-between rounded-2xl border p-3 text-left transition-all duration-200",
              "backdrop-blur-md backdrop-saturate-150",
              item.is_dir
                ? "border-emerald-300/40 bg-emerald-400/15 hover:bg-emerald-400/25 dark:border-emerald-500/20 dark:bg-emerald-500/10"
                : "border-sky-300/40 bg-sky-400/15 hover:bg-sky-400/25 dark:border-sky-500/20 dark:bg-sky-500/10",
              selected &&
                "ring-2 ring-emerald-500/60 ring-offset-2 ring-offset-transparent shadow-lg shadow-emerald-500/10",
            )}
            style={{ flex: `${minWidth} 1 120px` }}
            title={item.path}
          >
            <span className="line-clamp-2 text-xs font-semibold text-zinc-900 dark:text-zinc-100">
              {item.name}
            </span>
            <span className={cn("text-[11px]", text.muted)}>
              {formatBytes(size)}
            </span>
          </button>
        );
      })}
    </div>
  );
}
