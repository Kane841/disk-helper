import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { mockApi } from "@/mocks/mock-api";
import { api, useMockApi } from "@/lib/api";
import { useScanStore, useToastStore } from "@/stores/app-store";
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

interface ScanProgressPayload {
  scan_run_id: string;
  percent: number;
  scanned_files: number;
  skipped_files: number;
}

interface ScanCompletedPayload {
  scan_run_id: string;
  status: string;
  scanned_files: number;
  duration_ms: number;
}

interface ScanStatusBarProps {
  lastCompletedAt: string | null;
}

function toScanStatus(status: string): ScanStatus {
  if (
    status === "idle" ||
    status === "running" ||
    status === "paused" ||
    status === "completed" ||
    status === "failed"
  ) {
    return status;
  }
  if (status === "cancelled") return "idle";
  return "failed";
}

function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return fallback;
}

export function ScanStatusBar({ lastCompletedAt }: ScanStatusBarProps) {
  const queryClient = useQueryClient();
  const showToast = useToastStore((s) => s.show);
  const { status, progress, scannedFiles, setStatus, setProgress, setScannedFiles } =
    useScanStore();
  const timerRef = useRef<number | null>(null);

  const syncFromBackend = async () => {
    const snapshot = await api.scanGetStatus();
    setStatus(snapshot.status);
    setProgress(snapshot.progress_percent);
    setScannedFiles(snapshot.scanned_files);
  };

  useEffect(() => {
    return () => {
      if (timerRef.current) window.clearInterval(timerRef.current);
    };
  }, []);

  useEffect(() => {
    if (useMockApi) return;

    syncFromBackend().catch((error: Error) => {
      showToast(error.message);
    });

    let disposed = false;
    const unsubs: Array<() => void> = [];

    (async () => {
      unsubs.push(
        await listen<ScanProgressPayload>("scan://progress", (event) => {
          if (disposed) return;
          setStatus("running");
          setProgress(event.payload.percent);
          setScannedFiles(event.payload.scanned_files);
        }),
      );
      unsubs.push(
        await listen<ScanCompletedPayload>("scan://completed", (event) => {
          if (disposed) return;
          setStatus(toScanStatus(event.payload.status));
          setProgress(event.payload.status === "completed" ? 100 : useScanStore.getState().progress);
          setScannedFiles(event.payload.scanned_files);
          queryClient.invalidateQueries({ queryKey: ["scan-status"] });
          queryClient.invalidateQueries({ queryKey: ["categories"] });
          queryClient.invalidateQueries({ queryKey: ["index-children"] });
          queryClient.invalidateQueries({ queryKey: ["top-files"] });
          queryClient.invalidateQueries({ queryKey: ["top-folders"] });
        }),
      );
    })();

    return () => {
      disposed = true;
      unsubs.forEach((unsub) => unsub());
    };
  }, [queryClient, setProgress, setScannedFiles, setStatus, showToast]);

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

  const handleStart = async (type: "full" | "incremental" = "full") => {
    if (useMockApi) {
      setStatus("running");
      setProgress(0);
      setScannedFiles(0);
      mockApi.setScanStatus("running");
      startTimer();
      return;
    }

    try {
      await api.scanStart(type);
      setStatus("running");
      setProgress(0);
      setScannedFiles(0);
    } catch (error) {
      showToast(errorMessage(error, "启动扫描失败"));
    }
  };

  const handlePause = async () => {
    if (useMockApi) {
      if (timerRef.current) window.clearInterval(timerRef.current);
      setStatus("paused");
      mockApi.setScanStatus("paused");
      return;
    }

    try {
      await api.scanPause();
      setStatus("paused");
    } catch (error) {
      showToast(error instanceof Error ? error.message : "暂停失败");
    }
  };

  const handleResume = async () => {
    if (useMockApi) {
      setStatus("running");
      mockApi.setScanStatus("running");
      startTimer();
      return;
    }

    try {
      await api.scanResume();
      setStatus("running");
    } catch (error) {
      showToast(error instanceof Error ? error.message : "继续扫描失败");
    }
  };

  const handleCancel = async () => {
    if (!window.confirm("确定取消本次扫描？")) return;

    if (useMockApi) {
      if (timerRef.current) window.clearInterval(timerRef.current);
      setStatus("idle");
      setProgress(0);
      mockApi.setScanStatus("idle");
      return;
    }

    try {
      await api.scanCancel();
      setStatus("idle");
      setProgress(0);
      setScannedFiles(0);
    } catch (error) {
      showToast(error instanceof Error ? error.message : "取消失败");
    }
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
                <Button onClick={() => handleStart("full")}>开始扫描</Button>
                <Button variant="secondary" onClick={() => handleStart("incremental")}>
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
