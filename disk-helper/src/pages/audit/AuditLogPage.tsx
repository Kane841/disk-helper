import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { api } from "@/lib/api";
import { EVENT_TYPE_LABELS, formatDateTime } from "@/lib/format";
import { useToastStore } from "@/stores/app-store";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/button";
import { Card, CardBody } from "@/components/ui/card";

export function AuditLogPage() {
  const showToast = useToastStore((s) => s.show);
  const { data: logs = [] } = useQuery({
    queryKey: ["audit"],
    queryFn: () => api.auditList(),
  });

  return (
    <div className="flex h-full flex-col overflow-auto">
      <PageHeader title="操作日志">
        <Link
          to="/settings"
          className="text-sm font-medium text-emerald-600 hover:underline dark:text-emerald-400"
        >
          ← 返回设置
        </Link>
      </PageHeader>
      <div className="space-y-4 p-6">
        <Button
          variant="secondary"
          size="sm"
          onClick={() => showToast("诊断摘要已导出（模拟）")}
        >
          导出诊断摘要
        </Button>
        <Card strong>
          <CardBody className="p-0">
            <table className="w-full text-sm">
              <thead className={cn("text-left text-xs", glass.tableHead)}>
                <tr>
                  <th className="px-4 py-3">时间</th>
                  <th className="px-4 py-3">类型</th>
                  <th className="px-4 py-3">摘要</th>
                  <th className="px-4 py-3">结果</th>
                </tr>
              </thead>
              <tbody>
                {logs.map((log) => (
                  <tr
                    key={log.id}
                    className="border-b border-white/20 dark:border-white/5"
                  >
                    <td className="whitespace-nowrap px-4 py-3">
                      {formatDateTime(log.occurred_at)}
                    </td>
                    <td className="px-4 py-3">
                      {EVENT_TYPE_LABELS[log.event_type] ?? log.event_type}
                    </td>
                    <td className="max-w-md px-4 py-3">{log.summary}</td>
                    <td className="px-4 py-3">{log.result}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}
