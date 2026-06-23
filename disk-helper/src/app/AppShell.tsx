import { NavLink, Outlet } from "react-router-dom";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";
import { Toast } from "@/components/Toast";

const navItems = [
  { to: "/overview", label: "总览", icon: "◉" },
  { to: "/explorer", label: "浏览", icon: "▤" },
  { to: "/cleanup", label: "清理", icon: "✦" },
  { to: "/analysis", label: "分析", icon: "✧" },
  { to: "/settings", label: "设置", icon: "⚙" },
];

export function AppShell() {
  return (
    <div className={cn("app-shell-bg flex h-screen overflow-hidden", text.body)}>
      <aside className={cn("glass-sidebar relative z-10 flex w-60 shrink-0 flex-col")}>
        <div className="border-b px-5 py-6 glass-divider">
          <div className="flex items-center gap-2">
            <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-emerald-500/20 text-lg ring-1 ring-emerald-500/30 backdrop-blur-sm">
              💾
            </div>
            <div>
              <div className="text-[10px] font-semibold uppercase tracking-[0.2em] text-emerald-600 dark:text-emerald-300">
                Disk Helper
              </div>
              <h1 className={cn("text-base font-semibold tracking-tight", text.primary)}>磁盘助手</h1>
            </div>
          </div>
          <p className={cn("mt-3 text-[11px]", text.muted)}>
            智能空间管理 · UI 原型
          </p>
        </div>
        <nav className="flex-1 space-y-1 p-3">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition-all duration-200",
                  isActive ? glass.navActive : glass.navIdle,
                )
              }
            >
              <span className={cn("flex h-7 w-7 items-center justify-center rounded-lg bg-white/30 text-sm dark:bg-white/10 dark:text-zinc-200", text.secondary)}>
                {item.icon}
              </span>
              {item.label}
            </NavLink>
          ))}
          <NavLink
            to="/quarantine"
            className={({ isActive }) =>
              cn(
                "ml-4 flex items-center gap-2 rounded-lg px-3 py-2 text-xs transition-colors",
                isActive
                  ? "font-medium text-emerald-700 dark:text-emerald-300"
                  : cn(text.muted, "hover:text-zinc-800 dark:hover:text-zinc-100"),
              )
            }
          >
            → 隔离区
          </NavLink>
        </nav>
        <div className={cn("border-t p-4 text-[10px] glass-divider", text.faint)}>
          Mock 数据演示
        </div>
      </aside>
      <main className="relative z-10 flex min-w-0 flex-1 flex-col overflow-hidden">
        <Outlet />
      </main>
      <Toast />
    </div>
  );
}
