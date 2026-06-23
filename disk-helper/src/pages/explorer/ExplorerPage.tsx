import { useCallback, useEffect, useMemo, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { api, useMockApi } from "@/lib/api";
import { mockFileTree } from "@/mocks/fixtures";
import { useSelectionStore } from "@/stores/app-store";
import type { FileNode } from "@/types";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";
import { DiskTreemap } from "@/components/DiskTreemap";
import { FileDetailBar, FileTree } from "@/components/FileTree";
import { PageHeader } from "@/components/PageHeader";
import { RiskBadge } from "@/components/RiskBadge";
import { Button, GlassInput } from "@/components/ui/button";
import { Card, CardBody, CardHeader } from "@/components/ui/card";

type RightView = "treemap" | "top_files" | "top_folders" | "search";

const ROOT_PATH = "C:\\";

function buildTree(path: string, childrenMap: Record<string, FileNode[]>): FileNode[] {
  return (childrenMap[path] ?? []).map((node) => ({
    ...node,
    children: node.is_dir ? buildTree(node.path, childrenMap) : undefined,
  }));
}

export function ExplorerPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const setFromBrowse = useSelectionStore((s) => s.setFromBrowse);
  const [selectedPath, setSelectedPath] = useState(ROOT_PATH);
  const [selectedNode, setSelectedNode] = useState<FileNode>(
    useMockApi
      ? mockFileTree[0]
      : {
          path: ROOT_PATH,
          name: "C:\\",
          is_dir: true,
          size_bytes: 0,
          folder_size: 0,
        },
  );
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(() => new Set([ROOT_PATH]));
  const [rightView, setRightView] = useState<RightView>("treemap");
  const [searchKeyword, setSearchKeyword] = useState("");
  const [searchResults, setSearchResults] = useState<FileNode[]>([]);
  const [childrenMap, setChildrenMap] = useState<Record<string, FileNode[]>>({});
  const [loadedPaths, setLoadedPaths] = useState<Set<string>>(new Set());

  const mergeChildren = useCallback((path: string, nodes: FileNode[]) => {
    setChildrenMap((prev) => ({ ...prev, [path]: nodes }));
    setLoadedPaths((prev) => new Set(prev).add(path));
  }, []);

  const loadChildren = useCallback(
    async (path: string) => {
      const nodes = await api.indexGetChildren(path);
      mergeChildren(path, nodes);
      return nodes;
    },
    [mergeChildren],
  );

  const { isLoading: rootLoading, isError: rootError } = useQuery({
    queryKey: ["index-children", ROOT_PATH],
    queryFn: () => loadChildren(ROOT_PATH),
    enabled: !useMockApi,
    retry: false,
  });

  const { data: treemapChildren = [] } = useQuery({
    queryKey: ["index-children", selectedPath],
    queryFn: () => loadChildren(selectedPath),
    enabled: !useMockApi && selectedNode.is_dir && rightView === "treemap",
  });

  const { data: topFiles } = useQuery({
    queryKey: ["top-files", ROOT_PATH],
    queryFn: () => api.indexGetTopFiles(ROOT_PATH),
    enabled: !useMockApi && rightView === "top_files",
  });

  const { data: topFolders } = useQuery({
    queryKey: ["top-folders", ROOT_PATH],
    queryFn: () => api.indexGetTopFolders(ROOT_PATH),
    enabled: !useMockApi && rightView === "top_folders",
  });

  useEffect(() => {
    if (useMockApi) return;

    const invalidate = () => {
      queryClient.invalidateQueries({ queryKey: ["index-children"] });
      queryClient.invalidateQueries({ queryKey: ["top-files"] });
      queryClient.invalidateQueries({ queryKey: ["top-folders"] });
      setChildrenMap({});
      setLoadedPaths(new Set());
      loadChildren(ROOT_PATH).catch(() => undefined);
    };

    const unlistenPromise = listen("scan://completed", invalidate);

    return () => {
      unlistenPromise.then((unlisten) => unlisten()).catch(() => undefined);
    };
  }, [loadChildren, queryClient]);

  const treeNodes = useMockApi
    ? mockFileTree
    : [
        {
          path: ROOT_PATH,
          name: "C:\\",
          is_dir: true,
          size_bytes: 0,
          folder_size: 0,
          children: buildTree(ROOT_PATH, childrenMap),
        },
      ];
  const treemapItems = useMockApi
    ? (selectedNode.children ?? [])
    : selectedNode.is_dir
      ? treemapChildren
      : [];

  const handleSelect = async (node: FileNode) => {
    setSelectedPath(node.path);
    setSelectedNode(node);
    setFromBrowse(node);
    if (node.is_dir) {
      setExpandedPaths((prev) => new Set(prev).add(node.path));
      if (!useMockApi && !loadedPaths.has(node.path)) {
        try {
          await loadChildren(node.path);
        } catch {
          // keep selection; tree shows empty until retry
        }
      }
    }
  };

  const handleToggle = async (path: string) => {
    const willExpand = !expandedPaths.has(path);
    setExpandedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });

    if (!useMockApi && willExpand && !loadedPaths.has(path)) {
      try {
        await loadChildren(path);
      } catch {
        // ignore
      }
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

  const emptyHint = useMemo(() => {
    if (useMockApi) return null;
    if (rootLoading) {
      return <p className={cn("p-6 text-sm", text.muted)}>加载索引…</p>;
    }
    if (rootError || (loadedPaths.has(ROOT_PATH) && (childrenMap[ROOT_PATH]?.length ?? 0) === 0)) {
      return <p className={cn("p-6 text-sm", text.muted)}>请先完成全盘扫描以浏览索引数据</p>;
    }
    return null;
  }, [childrenMap, loadedPaths, rootError, rootLoading, useMockApi]);

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
          {emptyHint ?? (
            <FileTree
              nodes={treeNodes}
              selectedPath={selectedPath}
              expandedPaths={expandedPaths}
              loadedPaths={loadedPaths}
              lazy={!useMockApi}
              onSelect={handleSelect}
              onToggle={handleToggle}
            />
          )}
        </div>
        <div className="flex min-w-0 flex-1 flex-col gap-3">
          <Card className="flex min-h-0 flex-1 flex-col border-0 shadow-none" strong>
            <CardHeader className="py-3">
              <span className={cn("text-sm font-medium", text.secondary)}>
                {rightView === "treemap" && "Treemap · 当前目录子项"}
                {rightView === "top_files" && "大文件 Top"}
                {rightView === "top_folders" && "大文件夹 Top"}
                {rightView === "search" && `搜索结果 (${searchResults.length})`}
              </span>
            </CardHeader>
            <CardBody className="min-h-0 flex-1 overflow-auto p-0">
              {rightView === "treemap" && (
                <DiskTreemap
                  items={treemapItems}
                  selectedPath={selectedPath}
                  onSelect={handleSelect}
                />
              )}
              {(rightView === "top_files" || rightView === "top_folders") && (
                <table className="w-full text-sm">
                  <thead className={cn("text-left text-xs", glass.tableHead)}>
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
                      <p className={cn("truncate text-xs", text.muted)}>{item.path}</p>
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
