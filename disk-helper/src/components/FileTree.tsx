import type { FileNode } from "@/types";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/cn";
import { text } from "@/lib/theme";
import { RiskBadge } from "@/components/RiskBadge";

interface FileTreeProps {
  nodes: FileNode[];
  selectedPath: string;
  expandedPaths: Set<string>;
  onSelect: (node: FileNode) => void;
  onToggle: (path: string) => void;
  level?: number;
}

export function FileTree({
  nodes,
  selectedPath,
  expandedPaths,
  onSelect,
  onToggle,
  level = 0,
}: FileTreeProps) {
  return (
    <ul className={level === 0 ? "space-y-0.5" : "ml-3 border-l border-white/30 pl-2 dark:border-white/10"}>
      {nodes.map((node) => {
        const hasChildren = node.is_dir && (node.children?.length ?? 0) > 0;
        const expanded = expandedPaths.has(node.path);
        const selected = selectedPath === node.path;

        return (
          <li key={node.path}>
            <div
              className={cn(
                "flex cursor-pointer items-center gap-1 rounded-xl px-2 py-1.5 text-sm transition-all",
                selected
                  ? "glass-nav-active text-emerald-900 dark:text-emerald-200"
                  : "hover:bg-white/40 dark:hover:bg-white/5",
              )}
              onClick={() => onSelect(node)}
            >
              {node.is_dir ? (
                <button
                  type="button"
                  className={cn("w-4 shrink-0 text-xs", text.faint)}
                  onClick={(e) => {
                    e.stopPropagation();
                    if (hasChildren) onToggle(node.path);
                  }}
                >
                  {hasChildren ? (expanded ? "▼" : "▶") : "·"}
                </button>
              ) : (
                <span className={cn("w-4 shrink-0 text-center text-xs", text.faint)}>•</span>
              )}
              <span className="min-w-0 flex-1 truncate font-medium">{node.name}</span>
              <span className={cn("shrink-0 text-[11px]", text.muted)}>
                {formatBytes(node.is_dir ? node.folder_size : node.size_bytes)}
              </span>
            </div>
            {hasChildren && expanded && node.children && (
              <FileTree
                nodes={node.children}
                selectedPath={selectedPath}
                expandedPaths={expandedPaths}
                onSelect={onSelect}
                onToggle={onToggle}
                level={level + 1}
              />
            )}
          </li>
        );
      })}
    </ul>
  );
}

export function FileDetailBar({ node }: { node: FileNode | null }) {
  if (!node) {
    return (
      <div className={cn("glass-header px-4 py-3 text-sm", text.muted)}>
        选中一项以查看详情
      </div>
    );
  }
  return (
    <div
      className={cn(
        "glass-header flex flex-wrap items-center gap-x-6 gap-y-2 px-4 py-3 text-sm",
      )}
    >
      <span className="font-medium">{node.name}</span>
      <span className={text.muted}>
        {formatBytes(node.is_dir ? node.folder_size : node.size_bytes)}
      </span>
      {node.risk && <RiskBadge risk={node.risk} />}
      <span className={cn("min-w-0 flex-1 truncate text-xs", text.faint)}>{node.path}</span>
    </div>
  );
}
