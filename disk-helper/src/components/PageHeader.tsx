import { cn } from "@/lib/cn";

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
          <h2 className="text-xl font-semibold tracking-tight text-zinc-900 dark:text-zinc-50">
            {title}
          </h2>
          {description && (
            <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">{description}</p>
          )}
        </div>
        {children}
      </div>
    </header>
  );
}
