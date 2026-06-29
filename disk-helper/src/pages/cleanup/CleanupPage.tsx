import { useMemo, useState } from "react";
import { keepPreviousData, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { api } from "@/lib/api";
import { useDebouncedValue } from "@/lib/useDebouncedValue";
import { useSelectionStore, useToastStore } from "@/stores/app-store";
import type { CleanupSuggestion, RiskLevel } from "@/types";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";
import { GlassModal } from "@/components/GlassModal";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { PageHeader } from "@/components/PageHeader";
import { RiskBadge } from "@/components/RiskBadge";
import { Button, GlassInput, GlassSelect } from "@/components/ui/button";
import { Card, CardBody } from "@/components/ui/card";

const PAGE_SIZE = 50;

export function CleanupPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const showToast = useToastStore((s) => s.show);
  const setFromCleanup = useSelectionStore((s) => s.setFromCleanup);
  const [riskFilter, setRiskFilter] = useState<RiskLevel | "all">("all");
  const [pathKeyword, setPathKeyword] = useState("");
  const [page, setPage] = useState(1);
  const debouncedKeyword = useDebouncedValue(pathKeyword, 300);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [dangerUnlocked, setDangerUnlocked] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [confirmText, setConfirmText] = useState("");

  const { data, isLoading, isFetching } = useQuery({
    queryKey: ["suggestions", riskFilter, debouncedKeyword, page],
    queryFn: () =>
      api.rulesGetSuggestions({
        risk: riskFilter,
        path_keyword: debouncedKeyword || undefined,
        page,
        size: PAGE_SIZE,
      }),
    placeholderData: keepPreviousData,
    staleTime: 30_000,
  });

  const safePaths = useMemo(
    () => new Set(data?.items.filter((i) => i.risk === "safe").map((i) => i.path)),
    [data],
  );

  const pageCount = Math.max(1, Math.ceil((data?.total ?? 0) / PAGE_SIZE));

  const toggle = (path: string, item: CleanupSuggestion) => {
    if (item.risk === "danger" && !dangerUnlocked) return;
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const selectedSize = useMemo(() => {
    if (!data) return 0;
    return data.items
      .filter((i) => selected.has(i.path))
      .reduce((s, i) => s + i.size_bytes, 0);
  }, [data, selected]);

  const hasDangerSelected = data?.items.some(
    (i) => selected.has(i.path) && i.risk === "danger",
  );

  const executeCleanup = async () => {
    if (hasDangerSelected && confirmText !== "确认清理") {
      showToast("请输入「确认清理」以继续");
      return;
    }
    const paths = Array.from(selected);
    try {
      const result = await api.cleanupExecute({
        items: paths.map((path) => ({ path })),
        target: "quarantine",
        dangerConfirmToken: hasDangerSelected ? confirmText : undefined,
      });
      setSelected(new Set());
      setConfirmOpen(false);
      setConfirmText("");
      queryClient.invalidateQueries({ queryKey: ["quarantine"] });
      queryClient.invalidateQueries({ queryKey: ["audit"] });
      queryClient.invalidateQueries({ queryKey: ["suggestions"] });
      queryClient.invalidateQueries({ queryKey: ["categories"] });
      if (result.failed.length > 0) {
        showToast(
          `完成 ${result.success_count} 项，${result.failed.length} 项失败：${result.failed[0]?.reason ?? ""}`,
        );
      } else {
        showToast(`已移入隔离区 ${result.success_count} 项，释放约 ${formatBytes(selectedSize)}`);
      }
    } catch (error) {
      showToast(error instanceof Error ? error.message : "清理失败");
    }
  };

  return (
    <div className="flex h-full flex-col overflow-auto">
      <PageHeader
        title="安全清理"
        description={`预计可释放（安全项）：${formatBytes(data?.releasable_bytes ?? 0)}${
          data?.total != null ? ` · 共 ${data.total} 项` : ""
        }`}
      >
        <Button variant="secondary" onClick={() => navigate("/quarantine")}>
          跳转隔离区
        </Button>
      </PageHeader>
      <div className="space-y-4 p-6">
        <div className={cn(glass.panel, "flex flex-wrap items-center gap-3 p-4")}>
          <GlassSelect
            value={riskFilter}
            onChange={(e) => {
              setRiskFilter(e.target.value as RiskLevel | "all");
              setPage(1);
            }}
          >
            <option value="all">全部风险</option>
            <option value="safe">安全</option>
            <option value="caution">谨慎</option>
            <option value="danger">危险</option>
          </GlassSelect>
          <GlassInput
            placeholder="路径关键词"
            value={pathKeyword}
            onChange={(e) => {
              setPathKeyword(e.target.value);
              setPage(1);
            }}
          />
          <label className={cn("flex cursor-pointer items-center gap-2 text-sm", text.secondary)}>
            <input
              type="checkbox"
              checked={dangerUnlocked}
              onChange={(e) => setDangerUnlocked(e.target.checked)}
            />
            我了解风险
          </label>
          {isFetching && !isLoading && (
            <span className={cn("text-xs", text.muted)}>更新中…</span>
          )}
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="secondary" size="sm" onClick={() => setSelected(new Set(safePaths))}>
            全选安全项
          </Button>
          <Button variant="ghost" size="sm" onClick={() => setSelected(new Set())}>
            取消全选
          </Button>
          <Button size="sm" disabled={selected.size === 0} onClick={() => setConfirmOpen(true)}>
            执行清理 ({selected.size})
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              const items =
                data?.items.filter((i) => selected.has(i.path) || i.risk === "safe") ?? [];
              setFromCleanup(
                items.slice(0, 5).map((i) => ({
                  path: i.path,
                  size_bytes: i.size_bytes,
                  is_dir: i.is_dir,
                  risk: i.risk,
                  rule_description: i.description,
                })),
              );
              navigate("/analysis");
            }}
          >
            问 AI 解读清单
          </Button>
        </div>
        <Card strong>
          <CardBody className="p-0">
            {isLoading ? (
              <LoadingSpinner label="正在分析可清理项…" />
            ) : !data?.items.length ? (
              <p className={cn("p-12 text-center text-sm", text.muted)}>
                暂无匹配的清理建议，请先完成扫描或调整筛选条件
              </p>
            ) : (
              <>
                <table className="w-full text-sm">
                  <thead className={cn("text-left text-xs", glass.tableHead)}>
                    <tr>
                      <th className="w-10 px-4 py-3" />
                      <th className="px-4 py-3">路径</th>
                      <th className="px-4 py-3">大小</th>
                      <th className="px-4 py-3">风险</th>
                      <th className="px-4 py-3">说明</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.items.map((item) => (
                      <tr
                        key={item.path}
                        className="border-b border-white/20 dark:border-white/5"
                      >
                        <td className="px-4 py-3">
                          <input
                            type="checkbox"
                            checked={selected.has(item.path)}
                            disabled={item.risk === "danger" && !dangerUnlocked}
                            onChange={() => toggle(item.path, item)}
                          />
                        </td>
                        <td className="max-w-md truncate px-4 py-3" title={item.path}>
                          {item.path}
                        </td>
                        <td className="px-4 py-3">{formatBytes(item.size_bytes)}</td>
                        <td className="px-4 py-3">
                          <RiskBadge risk={item.risk} />
                        </td>
                        <td className={cn("max-w-xs px-4 py-3", text.muted)}>{item.description}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {pageCount > 1 && (
                  <div className="flex items-center justify-between border-t border-white/20 px-4 py-3 text-sm dark:border-white/5">
                    <span className={text.muted}>
                      第 {page} / {pageCount} 页
                    </span>
                    <div className="flex gap-2">
                      <Button
                        variant="secondary"
                        size="sm"
                        disabled={page <= 1 || isFetching}
                        onClick={() => setPage((p) => Math.max(1, p - 1))}
                      >
                        上一页
                      </Button>
                      <Button
                        variant="secondary"
                        size="sm"
                        disabled={page >= pageCount || isFetching}
                        onClick={() => setPage((p) => Math.min(pageCount, p + 1))}
                      >
                        下一页
                      </Button>
                    </div>
                  </div>
                )}
              </>
            )}
          </CardBody>
        </Card>
      </div>
      <GlassModal
        open={confirmOpen}
        onClose={() => setConfirmOpen(false)}
        title="确认清理"
      >
        <p className={cn("text-sm", text.secondary)}>
          将 {selected.size} 项移入隔离区，约 {formatBytes(selectedSize)}
        </p>
        {hasDangerSelected && (
          <>
            <p className="mt-3 text-sm text-red-600 dark:text-red-400">包含危险项，请输入「确认清理」</p>
            <GlassInput
              className="mt-2 w-full"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
            />
          </>
        )}
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="secondary" onClick={() => setConfirmOpen(false)}>
            取消
          </Button>
          <Button onClick={executeCleanup}>确认清理</Button>
        </div>
      </GlassModal>
    </div>
  );
}
