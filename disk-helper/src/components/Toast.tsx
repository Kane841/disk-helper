import { useToastStore } from "@/stores/app-store";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";

export function Toast() {
  const message = useToastStore((s) => s.message);
  if (!message) return null;
  return (
    <div className="pointer-events-none fixed bottom-6 left-1/2 z-50 -translate-x-1/2">
      <div
        className={cn(
          glass.panelStrong,
          "px-5 py-3 text-sm font-medium text-zinc-800 dark:text-zinc-100",
        )}
      >
        {message}
      </div>
    </div>
  );
}
