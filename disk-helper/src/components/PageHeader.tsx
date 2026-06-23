import { cn } from "@/lib/cn";
import { text } from "@/lib/theme";

export function PageHeader({
  title,
  description,
  children,
  className,
}: {
  title: string;
  description?: string;
  children?: React.ReactNode;
  className?: string;
}) {
  return (
    <header
      className={cn(
        "glass-header shrink-0 px-6 py-5",
        className,
      )}
    >
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h2 className={cn("text-xl font-semibold tracking-tight", text.primary)}>
            {title}
          </h2>
          {description && (
            <p className={cn("mt-1 text-sm", text.secondary)}>{description}</p>
          )}
        </div>
        {children}
      </div>
    </header>
  );
}
