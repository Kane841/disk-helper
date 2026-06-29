import { cn } from "@/lib/cn";

/** Shared glassmorphism class tokens */
export const glass = {
  panel: "glass-panel rounded-2xl",
  panelStrong: "glass-panel-strong rounded-2xl",
  header: "glass-header",
  input: "glass-input rounded-xl text-zinc-900 placeholder:text-zinc-400 dark:text-zinc-100 dark:placeholder:text-zinc-500",
  overlay: "glass-overlay",
  divider: "glass-divider",
  tableHead: "glass-table-head text-zinc-500 dark:text-zinc-300",
  navActive:
    "glass-nav-active text-emerald-800 ring-1 ring-white/50 dark:text-emerald-100 dark:ring-white/10",
  navIdle:
    "cursor-pointer text-zinc-600 hover:bg-white/35 hover:text-zinc-900 dark:text-zinc-200 dark:hover:bg-white/8 dark:hover:text-zinc-50",
} as const;

export function glassCn(...classes: (string | false | undefined | null)[]) {
  return cn(...classes);
}
