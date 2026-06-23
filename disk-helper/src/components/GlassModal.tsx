import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";

export function GlassModal({
  open,
  onClose,
  title,
  children,
  danger,
}: {
  open: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  danger?: boolean;
}) {
  if (!open) return null;
  return (
    <div
      className={cn("fixed inset-0 z-40 flex items-center justify-center p-4", glass.overlay)}
      onClick={onClose}
    >
      <div
        className={cn(glass.panelStrong, "w-full max-w-md p-6")}
        onClick={(e) => e.stopPropagation()}
      >
        <h3
          className={cn(
            "text-lg font-semibold tracking-tight",
            danger ? "text-red-600 dark:text-red-400" : text.primary,
          )}
        >
          {title}
        </h3>
        <div className={cn("mt-4", text.body)}>{children}</div>
      </div>
    </div>
  );
}
