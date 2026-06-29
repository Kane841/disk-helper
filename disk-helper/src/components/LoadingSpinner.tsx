import { cn } from "@/lib/cn";
import { text } from "@/lib/theme";

interface LoadingSpinnerProps {
  label?: string;
  className?: string;
}

export function LoadingSpinner({ label = "加载中…", className }: LoadingSpinnerProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center gap-3 py-12", className)}>
      <div className="loading-spinner h-8 w-8 rounded-full border-2 border-emerald-500/30 border-t-emerald-500" />
      <p className={cn("text-sm", text.muted)}>{label}</p>
    </div>
  );
}
