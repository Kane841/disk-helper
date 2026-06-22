import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { api } from "@/lib/api";
import { mockFileTree } from "@/mocks/fixtures";
import { useSelectionStore } from "@/stores/app-store";
import type { FileNode } from "@/types";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { DiskTreemap } from "@/components/DiskTreemap";
import { FileDetailBar, FileTree } from "@/components/FileTree";
import { PageHeader } from "@/components/PageHeader";
import { RiskBadge } from "@/components/RiskBadge";
import { Button, GlassInput } from "@/components/ui/button";
import { Card, CardBody, CardHeader } from "@/components/ui/card";

type RightView = "treemap" | "top_files" | "top_folders" | "search";

export function ExplorerPage() {
  const navigate = useNavigate();
  const setFromBrowse = useSelectionStore((s) => s.setFromBrowse);
  const [selectedPath, setSelectedPath] = useState("C:\\");
  const [selectedNode, setSelectedNode] = useState<FileNode>(mockFileTree[0]);
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(
    () => new Set(["C:\\", "C:\\Users\\AH"]),
  );
  const [rightView, setRightView] = useState<RightView>("treemap");
  const [searchKeyword, setSearchKeyword] = useState("");
  const [searchResults, setSearchResults] = useState<FileNode[]>([]);

  const children = selectedNode.children ?? [];

  const { data: topFiles } = useQuery({
    queryKey: ["top-files"],
    queryFn: () => api.indexGetTopFiles(),
    enabled: rightView === "top_files",
  });
  const { data: topFolders } = useQuery({
    queryKey: ["top-folders"],
    queryFn: () => api.indexGetTopFolders(),
    enabled: rightView === "top_folders",
  });

  const handleSelect = (node: FileNode) => {
    setSelectedPath(node.path);
    setSelectedNode(node);
    setFromBrowse(node);
    if (node.is_dir && node.children?.length) {
      setExpandedPaths((prev) => new Set(prev).add(node.path));
    }
  };

  const handleSearch = async () => {
    if (!searchKeyword.trim()) {
      setRightView("treemap");
      return;
    }
    const results = await api.indexSearch(searchKeyword);
    setSearchResults(results);
    setRightView("search");
  };

  return (
    <div className="flex h-full flex-col">
      <PageHeader title="空间浏览" description={selectedPath}>
        <div className="flex flex-wrap items-center gap-2">
          <GlassInput
            className="w-48"
            placeholder="搜索路径或文件名"
            value={searchKeyword}
            onChange={(e) => setSearchKeyword(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          />
          <Button variant="secondary" size="sm" onClick={handleSearch}>
            搜索
          </Button>
          <Button variant="secondary" size="sm" onClick={() => setRightView("top_files")}>
            大文件
          </Button>
          <Button variant="secondary" size="sm" onClick={() => setRightView("top_folders")}>
            大文件夹
          </Button>
          <Button variant="secondary" size="sm" onClick={() => setRightView("treemap")}>
            Treemap
          </Button>
          <Button size="sm" onClick={() => navigate("/analysis")}>
            问 AI
          </Button>
        </div>
      </PageHeader>
      <div className="flex min-h-0 flex-1 gap-3 p-3">
        <div className={cn(glass.panel, "w-72 shrink-0 overflow-auto p-3")}>
          <FileTree
            nodes={mockFileTree}
            selectedPath={selectedPath}
            expandedPaths={expandedPaths}
            onSelect={handleSelect}
            onToggle={(path) =>
              setExpandedPaths((prev) => {
                const next = new Set(prev);
                if (next.has(path)) next.delete(path);
                else next.add(path);
                return next;
              })
            }
          />
        </div>
        <div className="flex min-w-0 flex-1 flex-col gap-3">
          <Card className="flex min-h-0 flex-1 flex-col border-0 shadow-none" strong>
            <CardHeader className="py-3">
              <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
                {rightView === "treemap" && "Treemap · 当前目录子项"}
                {rightView === "top_files" && "大文件 Top"}
                {rightView === "top_folders" && "大文件夹 Top"}
                {rightView === "search" && `搜索结果 (${searchResults.length})`}
              </span>
            </CardHeader>
            <CardBody className="min-h-0 flex-1 overflow-auto p-0">
              {rightView === "treemap" && (
                <DiskTreemap
                  items={children}
                  selectedPath={selectedPath}
                  onSelect={handleSelect}
                />
              )}
              {(rightView === "top_files" || rightView === "top_folders") && (
                <table className="w-full text-sm">
                  <thead className={cn("text-left text-xs text-zinc-500", glass.tableHead)}>
                    <tr>
                      <th className="px-4 py-2">名称</th>
                      <th className="px-4 py-2">大小</th>
                      <th className="px-4 py-2">风险</th>
                    </tr>
                  </thead>
                  <tbody>
                    {(rightView === "top_files" ? topFiles : topFolders)?.map((item) => (
                      <tr
                        key={item.path}
                        className="cursor-pointer border-b border-white/20 hover:bg-white/30 dark:border-white/5 dark:hover:bg-white/5"
                        onClick={() => handleSelect(item)}
                      >
                        <td className="max-w-xs truncate px-4 py-2">{item.name}</td>
                        <td className="px-4 py-2">
                          {formatBytes(item.is_dir ? item.folder_size : item.size_bytes)}
                        </td>
                        <td className="px-4 py-2">
                          {item.risk && <RiskBadge risk={item.risk} />}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
              {rightView === "search" && (
                <ul className="divide-y divide-white/25 dark:divide-white/5">
                  {searchResults.map((item) => (
                    <li
                      key={item.path}
                      className="cursor-pointer px-4 py-3 hover:bg-white/30 dark:hover:bg-white/5"
                      onClick={() => handleSelect(item)}
                    >
                      <p className="font-medium">{item.name}</p>
                      <p className="truncate text-xs text-zinc-500">{item.path}</p>
                    </li>
                  ))}
                </ul>
              )}
            </CardBody>
          </Card>
          <FileDetailBar node={selectedNode} />
        </div>
      </div>
    </div>
  );
}
