import { useEffect, useRef } from "react";
import { mockApi } from "@/mocks/mock-api";
import { useScanStore } from "@/stores/app-store";
import type { ScanStatus } from "@/types";
import { formatDateTime } from "@/lib/format";
import { cn } from "@/lib/cn";
import { text } from "@/lib/theme";
import { Button } from "@/components/ui/button";
import { Card, CardBody } from "@/components/ui/card";

const STATUS_LABELS: Record<ScanStatus, string> = {
  idle: "未扫描",
  running: "扫描中",
  paused: "已暂停",
  completed: "已完成",
  failed: "失败",
};

interface ScanStatusBarProps {
  lastCompletedAt: string | null;
}

export function ScanStatusBar({ lastCompletedAt }: ScanStatusBarProps) {
  const { status, progress, scannedFiles, setStatus, setProgress, setScannedFiles } =
    useScanStore();
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current) window.clearInterval(timerRef.current);
    };
  }, []);

  const startTimer = () => {
    if (timerRef.current) window.clearInterval(timerRef.current);
    timerRef.current = window.setInterval(() => {
      const p = useScanStore.getState().progress;
      const next = Math.min(p + 2, 100);
      setProgress(next);
      setScannedFiles(Math.floor((next / 100) * 284521));
      if (next >= 100) {
        if (timerRef.current) window.clearInterval(timerRef.current);
        setStatus("completed");
        mockApi.setScanStatus("completed");
      }
    }, 120);
  };

  const handleStart = () => {
    setStatus("running");
    setProgress(0);
    setScannedFiles(0);
    mockApi.setScanStatus("running");
    startTimer();
  };

  const handlePause = () => {
    if (timerRef.current) window.clearInterval(timerRef.current);
    setStatus("paused");
    mockApi.setScanStatus("paused");
  };

  const handleResume = () => {
    setStatus("running");
    mockApi.setScanStatus("running");
    startTimer();
  };

  const handleCancel = () => {
    if (!window.confirm("确定取消本次扫描？")) return;
    if (timerRef.current) window.clearInterval(timerRef.current);
    setStatus("idle");
    setProgress(0);
    mockApi.setScanStatus("idle");
  };

  return (
    <Card>
      <CardBody>
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <p className={cn("text-xs font-semibold uppercase tracking-wider", text.muted)}>
              扫描状态
            </p>
            <p className={cn("mt-1 text-lg font-medium", text.primary)}>{STATUS_LABELS[status]}</p>
            {lastCompletedAt && status === "completed" && (
              <p className={cn("mt-1 text-xs", text.faint)}>
                上次完成：{formatDateTime(lastCompletedAt)}
              </p>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            {(status === "idle" || status === "completed" || status === "failed") && (
              <>
                <Button onClick={handleStart}>开始扫描</Button>
                <Button variant="secondary" onClick={handleStart}>
                  增量扫描
                </Button>
              </>
            )}
            {status === "running" && (
              <>
                <Button variant="secondary" onClick={handlePause}>
                  暂停
                </Button>
                <Button variant="ghost" onClick={handleCancel}>
                  取消
                </Button>
              </>
            )}
            {status === "paused" && (
              <>
                <Button onClick={handleResume}>继续</Button>
                <Button variant="ghost" onClick={handleCancel}>
                  取消
                </Button>
              </>
            )}
          </div>
        </div>
        {(status === "running" || status === "paused") && (
          <div className="mt-4">
            <div className={cn("mb-1 flex justify-between text-xs", text.muted)}>
              <span>{progress}%</span>
              <span>已扫描 {scannedFiles.toLocaleString()} 个文件</span>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-white/40 ring-1 ring-white/30 dark:bg-white/5">
              <div
                className="h-full rounded-full bg-gradient-to-r from-emerald-600 to-teal-400 transition-all"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}
      </CardBody>
    </Card>
  );
}
