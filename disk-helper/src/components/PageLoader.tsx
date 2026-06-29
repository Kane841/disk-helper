import { cn } from "@/lib/cn";
import { text } from "@/lib/theme";

export function PageLoader() {
  return (
    <div className="flex h-full items-center justify-center">
      <p className={cn("text-sm", text.muted)}>加载中…</p>
    </div>
  );
}
