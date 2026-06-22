import { cn } from "@/lib/cn";

/** Shared glassmorphism class tokens */
export const glass = {
  panel: "glass-panel rounded-2xl",
  panelStrong: "glass-panel-strong rounded-2xl",
  header: "glass-header",
  input: "glass-input rounded-xl text-zinc-900 placeholder:text-zinc-400 dark:text-zinc-100 dark:placeholder:text-zinc-500",
  overlay: "glass-overlay",
  divider: "glass-divider",
  tableHead: "glass-table-head",
  navActive:
    "glass-nav-active text-emerald-800 ring-1 ring-white/50 dark:text-emerald-300 dark:ring-white/10",
  navIdle:
    "text-zinc-600 hover:bg-white/35 dark:text-zinc-400 dark:hover:bg-white/5",
} as const;

export function glassCn(...classes: (string | false | undefined | null)[]) {
  return cn(...classes);
}
