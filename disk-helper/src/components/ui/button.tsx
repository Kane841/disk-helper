import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";

const variants = {
  default:
    "bg-emerald-600/90 text-white shadow-lg shadow-emerald-600/20 hover:bg-emerald-600 backdrop-blur-md border border-emerald-400/30",
  secondary:
    "glass-panel text-zinc-800 hover:bg-white/65 dark:text-zinc-100 dark:hover:bg-white/10 border-0",
  ghost:
    "text-zinc-700 hover:bg-white/40 dark:text-zinc-300 dark:hover:bg-white/5 backdrop-blur-sm",
  danger:
    "bg-red-500/90 text-white shadow-lg shadow-red-500/25 hover:bg-red-600 backdrop-blur-md border border-red-400/30",
};

const sizes = {
  sm: "h-8 px-3 text-xs rounded-xl",
  md: "h-9 px-4 text-sm rounded-xl",
  lg: "h-10 px-5 text-sm rounded-xl",
};

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: keyof typeof variants;
  size?: keyof typeof sizes;
}

export function Button({
  className,
  variant = "default",
  size = "md",
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center gap-2 font-medium transition-all duration-200 disabled:pointer-events-none disabled:opacity-50",
        variants[variant],
        sizes[size],
        className,
      )}
      {...props}
    />
  );
}

export function GlassSelect({
  className,
  ...props
}: React.SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select className={cn("h-9 px-3 text-sm", glass.input, className)} {...props} />
  );
}

export function GlassInput({
  className,
  ...props
}: React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input className={cn("h-9 px-3 text-sm", glass.input, className)} {...props} />
  );
}
