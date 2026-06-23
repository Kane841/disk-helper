import { useNavigate } from "react-router-dom";
import type { CategoryStat } from "@/types";
import { CATEGORY_LABELS, formatBytes } from "@/lib/format";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";

export function CategoryGrid({ categories }: { categories: CategoryStat[] }) {
  const navigate = useNavigate();

  return (
    <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
      {categories.map((cat) => (
        <button
          key={cat.code}
          type="button"
          onClick={() => navigate("/explorer")}
          className={cn(
            glass.panel,
            "group p-4 text-left transition-all duration-200 hover:scale-[1.01] hover:shadow-lg",
          )}
        >
          <div className="flex items-center justify-between">
            <span className={cn("text-sm font-medium", text.primary)}>
              {CATEGORY_LABELS[cat.code] ?? cat.code}
            </span>
            <span className={cn("text-xs", text.muted)}>{(cat.ratio * 100).toFixed(0)}%</span>
          </div>
          <p className={cn("mt-2 text-lg font-semibold", text.primary)}>{formatBytes(cat.size_bytes)}</p>
          <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-white/40 dark:bg-white/5">
            <div
              className="h-full rounded-full bg-gradient-to-r from-emerald-500/80 to-teal-400/80"
              style={{ width: `${cat.ratio * 100}%` }}
            />
          </div>
          <p className={cn("mt-2 text-[11px] opacity-0 transition-opacity group-hover:opacity-100", text.faint)}>
            点击查看详情 →
          </p>
        </button>
      ))}
    </div>
  );
}
