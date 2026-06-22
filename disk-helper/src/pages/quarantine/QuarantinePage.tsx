import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useToastStore } from "@/stores/app-store";
import { formatBytes, formatDateTime } from "@/lib/format";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { GlassModal } from "@/components/GlassModal";
import { PageHeader } from "@/components/PageHeader";
import { RiskBadge } from "@/components/RiskBadge";
import { Button, GlassInput } from "@/components/ui/button";
import { Card, CardBody } from "@/components/ui/card";

const STATUS_LABELS = {
  active: "有效",
  expiring: "即将过期",
  expired: "已过期",
};

export function QuarantinePage() {
  const queryClient = useQueryClient();
  const showToast = useToastStore((s) => s.show);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [purgeOpen, setPurgeOpen] = useState(false);
  const [purgeText, setPurgeText] = useState("");

  const { data: items = [] } = useQuery({
    queryKey: ["quarantine"],
    queryFn: () => api.quarantineList(),
  });

  const totalSize = items.reduce((s, i) => s + i.size_bytes, 0);

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleRestore = async () => {
    if (selected.size === 0) {
      showToast("请先勾选项目");
      return;
    }
    const count = selected.size;
    await api.quarantineRestore(Array.from(selected));
    setSelected(new Set());
    queryClient.invalidateQueries({ queryKey: ["quarantine"] });
    showToast(`已还原 ${count} 项（模拟）`);
  };

  const handlePurge = async () => {
    if (purgeText !== "永久删除") {
      showToast("请输入「永久删除」以继续");
      return;
    }
    await api.quarantinePurge(Array.from(selected));
    setSelected(new Set());
    setPurgeOpen(false);
    setPurgeText("");
    queryClient.invalidateQueries({ queryKey: ["quarantine"] });
    showToast("已永久删除（模拟）");
  };

  return (
    <div className="flex h-full flex-col overflow-auto">
      <PageHeader
        title="隔离区"
        description={`共 ${items.length} 项 · 占用 ${formatBytes(totalSize)} · 保留 30 天`}
      />
      <div className="space-y-4 p-6">
        <div className="flex flex-wrap gap-2">
          <Button size="sm" onClick={handleRestore}>
            还原选中
          </Button>
          <Button
            variant="danger"
            size="sm"
            disabled={selected.size === 0}
            onClick={() => setPurgeOpen(true)}
          >
            永久删除选中
          </Button>
        </div>
        <Card strong>
          <CardBody className="p-0">
            {items.length === 0 ? (
              <p className="p-12 text-center text-sm text-zinc-500">
                隔离区为空，清理的文件将显示在这里
              </p>
            ) : (
              <table className="w-full text-sm">
                <thead className={cn("text-left text-xs text-zinc-500", glass.tableHead)}>
                  <tr>
                    <th className="w-10 px-4 py-3" />
                    <th className="px-4 py-3">原路径</th>
                    <th className="px-4 py-3">大小</th>
                    <th className="px-4 py-3">移入时间</th>
                    <th className="px-4 py-3">状态</th>
                    <th className="px-4 py-3">风险</th>
                  </tr>
                </thead>
                <tbody>
                  {items.map((item) => (
                    <tr
                      key={item.id}
                      className="border-b border-white/20 dark:border-white/5"
                    >
                      <td className="px-4 py-3">
                        <input
                          type="checkbox"
                          checked={selected.has(item.id)}
                          onChange={() => toggle(item.id)}
                        />
                      </td>
                      <td className="max-w-md truncate px-4 py-3" title={item.original_path}>
                        {item.original_path}
                      </td>
                      <td className="px-4 py-3">{formatBytes(item.size_bytes)}</td>
                      <td className="px-4 py-3">{formatDateTime(item.moved_at)}</td>
                      <td className="px-4 py-3">{STATUS_LABELS[item.status]}</td>
                      <td className="px-4 py-3">
                        <RiskBadge risk={item.risk} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </CardBody>
        </Card>
      </div>
      <GlassModal
        open={purgeOpen}
        onClose={() => setPurgeOpen(false)}
        title="永久删除"
        danger
      >
        <p className="text-sm text-zinc-600 dark:text-zinc-400">此操作不可恢复</p>
        <GlassInput
          className="mt-3 w-full"
          placeholder="输入「永久删除」"
          value={purgeText}
          onChange={(e) => setPurgeText(e.target.value)}
        />
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="secondary" onClick={() => setPurgeOpen(false)}>
            取消
          </Button>
          <Button variant="danger" onClick={handlePurge}>
            确认永久删除
          </Button>
        </div>
      </GlassModal>
    </div>
  );
}
