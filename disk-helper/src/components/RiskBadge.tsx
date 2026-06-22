import type { RiskLevel } from "@/types";
import { RISK_LABELS } from "@/lib/format";
import { cn } from "@/lib/cn";

const styles: Record<RiskLevel, string> = {
  safe:
    "bg-emerald-500/15 text-emerald-800 ring-emerald-500/25 dark:bg-emerald-500/15 dark:text-emerald-300",
  caution:
    "bg-amber-500/15 text-amber-800 ring-amber-500/25 dark:bg-amber-500/15 dark:text-amber-300",
  danger:
    "bg-red-500/15 text-red-800 ring-red-500/25 dark:bg-red-500/15 dark:text-red-300",
};

export function RiskBadge({ risk }: { risk: RiskLevel }) {
  return (
    <span
      className={cn(
        "inline-flex rounded-lg px-2 py-0.5 text-xs font-medium ring-1 backdrop-blur-sm",
        styles[risk],
      )}
    >
      {RISK_LABELS[risk]}
    </span>
  );
}
