import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { api } from "@/lib/api";
import { useScanStore } from "@/stores/app-store";
import { CapacityCard } from "@/components/CapacityCard";
import { CategoryGrid } from "@/components/CategoryGrid";
import { PageHeader } from "@/components/PageHeader";
import { ScanStatusBar } from "@/components/ScanStatusBar";
import { Button } from "@/components/ui/button";
import { text } from "@/lib/theme";
import { cn } from "@/lib/cn";

export function OverviewPage() {
  const navigate = useNavigate();
  const scanStatus = useScanStore((s) => s.status);
  const { data: volume } = useQuery({
    queryKey: ["volume"],
    queryFn: () => api.volumeGetCDrive(),
  });
  const { data: categories, isError: categoriesError } = useQuery({
    queryKey: ["categories"],
    queryFn: () => api.indexGetCategoryStats(),
    retry: false,
  });
  const { data: scanInfo } = useQuery({
    queryKey: ["scan-status"],
    queryFn: () => api.scanGetStatus(),
  });

  return (
    <div className="flex h-full flex-col overflow-auto">
      <PageHeader
        title="磁盘总览"
        description="C 盘空间状况与扫描入口"
      />
      <div className="flex-1 space-y-6 p-6">
        {volume && <CapacityCard volume={volume} />}
        <ScanStatusBar lastCompletedAt={scanInfo?.last_completed_at ?? null} />
        {(scanStatus === "running" || scanStatus === "paused") && (
          <p className={cn("text-sm", text.muted)}>
            扫描进行中，分类占用与浏览数据将在完成后更新。
          </p>
        )}
        <div>
          <h3 className={cn("mb-3 text-xs font-semibold uppercase tracking-wider", text.muted)}>
            分类占用
          </h3>
          {categories && <CategoryGrid categories={categories} />}
          {!categories && categoriesError && (
            <p className={cn("text-sm", text.muted)}>完成首次扫描后将显示分类占用</p>
          )}
        </div>
        <div className="flex flex-wrap gap-3">
          <Button onClick={() => navigate("/explorer")}>查看空间浏览</Button>
          <Button variant="secondary" onClick={() => navigate("/cleanup")}>
            查看清理建议
          </Button>
          <Button variant="secondary" onClick={() => navigate("/analysis")}>
            问 AI
          </Button>
        </div>
      </div>
    </div>
  );
}
