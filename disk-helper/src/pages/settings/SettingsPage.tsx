import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { useTheme } from "next-themes";
import { api, useMockApi } from "@/lib/api";
import { TauriApiError } from "@/lib/tauri-client";
import { useToastStore } from "@/stores/app-store";
import type { AppSettings } from "@/types";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";
import { PageHeader } from "@/components/PageHeader";
import { Button, GlassInput } from "@/components/ui/button";
import { Card, CardBody } from "@/components/ui/card";
import { GlassModal } from "@/components/GlassModal";

type Tab = "ai" | "appearance" | "quarantine" | "scan" | "advanced";

export function SettingsPage() {
  const { setTheme } = useTheme();
  const queryClient = useQueryClient();
  const showToast = useToastStore((s) => s.show);
  const [tab, setTab] = useState<Tab>("ai");
  const [draft, setDraft] = useState<Partial<AppSettings>>({});
  const [apiKey, setApiKey] = useState("");
  const [testResult, setTestResult] = useState<string | null>(null);
  const [clearOpen, setClearOpen] = useState(false);
  const [clearConfirm, setClearConfirm] = useState("");
  const [clearing, setClearing] = useState(false);

  const { data: settings } = useQuery({
    queryKey: ["settings"],
    queryFn: () => api.configGet(),
  });

  const merged = { ...settings, ...draft } as AppSettings | undefined;
  const previewTheme = draft.theme ?? settings?.theme ?? "system";

  const update = (partial: Partial<AppSettings>) => {
    setDraft((d) => ({ ...d, ...partial }));
  };

  const save = async () => {
    const savedTheme = draft.theme ?? settings?.theme;
    try {
      await api.configSave({
        ...draft,
        api_key: apiKey.trim() || undefined,
      });
      setDraft({});
      setApiKey("");
      if (savedTheme) {
        setTheme(savedTheme);
      }
      queryClient.invalidateQueries({ queryKey: ["settings"] });
      showToast("设置已保存");
    } catch (error) {
      const message =
        error instanceof TauriApiError ? error.message : "保存设置失败";
      showToast(message);
    }
  };

  const testConnection = async () => {
    try {
      const res = await api.aiTestConnection({
        ai_mode: merged?.ai_mode,
        api_key: apiKey.trim() || undefined,
      });
      setTestResult(res.message);
    } catch (error) {
      const message =
        error instanceof TauriApiError ? error.message : "连接测试失败";
      setTestResult(message);
    }
  };

  const handleClearIndex = async () => {
    if (clearConfirm !== "清空索引") {
      showToast('请输入确认文字「清空索引」');
      return;
    }
    setClearing(true);
    try {
      const res = await api.indexClear(true);
      setClearOpen(false);
      setClearConfirm("");
      queryClient.invalidateQueries({ queryKey: ["categories"] });
      queryClient.invalidateQueries({ queryKey: ["index-children"] });
      queryClient.invalidateQueries({ queryKey: ["top-files"] });
      queryClient.invalidateQueries({ queryKey: ["top-folders"] });
      queryClient.invalidateQueries({ queryKey: ["suggestions"] });
      queryClient.invalidateQueries({ queryKey: ["scan-status"] });
      showToast(`已清空索引（${res.deleted_entries} 条记录）`);
    } catch (error) {
      const message =
        error instanceof TauriApiError ? error.message : "清空索引失败";
      showToast(message);
    } finally {
      setClearing(false);
    }
  };

  const tabs: { id: Tab; label: string }[] = [
    { id: "ai", label: "AI 配置" },
    { id: "appearance", label: "外观" },
    { id: "quarantine", label: "隔离区" },
    { id: "scan", label: "扫描" },
    { id: "advanced", label: "高级" },
  ];

  return (
    <div className="flex h-full flex-col overflow-auto">
      <PageHeader title="设置">
        <Link
          to="/settings/audit"
          className="text-sm font-medium text-emerald-600 hover:underline dark:text-emerald-400"
        >
          操作日志 →
        </Link>
      </PageHeader>
      <div className="flex flex-1 flex-col gap-6 p-6 lg:flex-row">
        <div className={cn(glass.panel, "flex gap-1 p-2 lg:flex-col lg:gap-1")}>
          {tabs.map((t) => (
            <button
              key={t.id}
              type="button"
              onClick={() => setTab(t.id)}
              className={cn(
                "rounded-xl px-4 py-2.5 text-left text-sm transition-all",
                tab === t.id ? glass.navActive : glass.navIdle,
              )}
            >
              {t.label}
            </button>
          ))}
        </div>
        <Card className="min-w-0 flex-1" strong>
          <CardBody className="space-y-4">
            {tab === "ai" && merged && (
              <>
                <div>
                  <label className={text.label}>AI 模式</label>
                  <div className="mt-3 flex gap-3">
                    {(["local", "cloud"] as const).map((mode) => (
                      <button
                        key={mode}
                        type="button"
                        onClick={() => update({ ai_mode: mode })}
                        className={cn(
                          "flex-1 rounded-xl px-4 py-3 text-sm font-medium transition-all",
                          merged.ai_mode === mode ? glass.navActive : glass.navIdle,
                        )}
                      >
                        {mode === "local" ? "本地 Ollama" : "云端 DeepSeek"}
                      </button>
                    ))}
                  </div>
                </div>
                {merged.ai_mode === "local" ? (
                  <>
                    <div>
                      <label className={text.label}>Ollama 地址</label>
                      <GlassInput
                        className="mt-2 w-full"
                        value={merged.ollama_base_url}
                        onChange={(e) => update({ ollama_base_url: e.target.value })}
                      />
                    </div>
                    <div>
                      <label className={text.label}>模型</label>
                      <GlassInput
                        className="mt-2 w-full opacity-70"
                        value={merged.ollama_model}
                        readOnly
                      />
                    </div>
                  </>
                ) : (
                  <div>
                    <label className={text.label}>API Key</label>
                    <GlassInput
                      type="password"
                      className="mt-2 w-full"
                      placeholder={
                        merged.has_api_key ? "已配置（输入新 Key 覆盖）" : "输入 DeepSeek API Key"
                      }
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                    />
                  </div>
                )}
                <Button variant="secondary" onClick={testConnection}>
                  测试连接
                </Button>
                {testResult && <p className={cn("text-sm", text.muted)}>{testResult}</p>}
              </>
            )}
            {tab === "appearance" && (
              <div>
                <label className={text.label}>界面主题</label>
                <div className="mt-3 flex gap-2">
                  {(["system", "light", "dark"] as const).map((t) => (
                    <button
                      key={t}
                      type="button"
                      onClick={() => {
                        setTheme(t);
                        update({ theme: t });
                      }}
                      className={cn(
                        "cursor-pointer rounded-xl px-4 py-2 text-sm capitalize",
                        previewTheme === t ? glass.navActive : glass.navIdle,
                      )}
                    >
                      {t === "system" ? "跟随系统" : t === "light" ? "浅色" : "深色"}
                    </button>
                  ))}
                </div>
              </div>
            )}
            {tab === "quarantine" && merged && (
              <>
                <div>
                  <label className={text.label}>隔离区路径</label>
                  <GlassInput
                    className="mt-2 w-full"
                    value={merged.quarantine_root}
                    onChange={(e) => update({ quarantine_root: e.target.value })}
                  />
                </div>
                <div>
                  <label className={text.label}>保留天数</label>
                  <GlassInput
                    type="number"
                    className="mt-2 w-32"
                    value={merged.retention_days}
                    onChange={(e) => update({ retention_days: Number(e.target.value) })}
                  />
                </div>
              </>
            )}
            {tab === "scan" && merged && (
              <>
                <label className={cn("flex items-center gap-2 text-sm", text.secondary)}>
                  <input
                    type="checkbox"
                    checked={merged.admin_scan_enabled}
                    onChange={(e) => update({ admin_scan_enabled: e.target.checked })}
                  />
                  启用管理员扫描
                </label>
                <div className="grid gap-4 sm:grid-cols-2">
                  <div>
                    <label className={text.label}>警告阈值 (GB)</label>
                    <GlassInput
                      type="number"
                      className="mt-2 w-full"
                      value={merged.warning_threshold_gb}
                      onChange={(e) => update({ warning_threshold_gb: Number(e.target.value) })}
                    />
                  </div>
                  <div>
                    <label className={text.label}>严重阈值 (GB)</label>
                    <GlassInput
                      type="number"
                      className="mt-2 w-full"
                      value={merged.critical_threshold_gb}
                      onChange={(e) => update({ critical_threshold_gb: Number(e.target.value) })}
                    />
                  </div>
                </div>
              </>
            )}
            {tab === "advanced" && merged && (
              <div className="space-y-6">
                <div>
                  <label className={text.label}>软删除默认目标</label>
                  <div className="mt-3 flex gap-2">
                    {(["quarantine", "recycle_bin"] as const).map((target) => (
                      <button
                        key={target}
                        type="button"
                        onClick={() => update({ soft_delete_target: target })}
                        className={cn(
                          "rounded-xl px-4 py-2 text-sm",
                          merged.soft_delete_target === target ? glass.navActive : glass.navIdle,
                        )}
                      >
                        {target === "quarantine" ? "隔离区" : "回收站"}
                      </button>
                    ))}
                  </div>
                </div>
                {!useMockApi && (
                  <div className="space-y-3 border-t border-white/25 pt-4 dark:border-white/10">
                    <label className={text.label}>清空扫描索引</label>
                    <p className={cn("text-sm", text.muted)}>
                      删除已扫描的文件索引与扫描记录，不影响隔离区与操作日志。清空后需重新全盘扫描。
                    </p>
                    <Button variant="secondary" onClick={() => setClearOpen(true)}>
                      清空索引…
                    </Button>
                  </div>
                )}
              </div>
            )}
            <div className="flex gap-2 border-t border-white/25 pt-4 dark:border-white/10">
              <Button onClick={save}>保存</Button>
              <Button
                variant="secondary"
                onClick={() => {
                  if (draft.theme) {
                    setTheme(settings?.theme ?? "system");
                  }
                  setDraft({});
                }}
              >
                重置未保存更改
              </Button>
            </div>
          </CardBody>
        </Card>
      </div>
      <GlassModal
        open={clearOpen}
        onClose={() => {
          setClearOpen(false);
          setClearConfirm("");
        }}
        title="确认清空扫描索引"
        danger
      >
        <p className={cn("text-sm", text.muted)}>
          此操作不可撤销。请输入「清空索引」以确认。
        </p>
        <GlassInput
          className="mt-4 w-full"
          placeholder="清空索引"
          value={clearConfirm}
          onChange={(e) => setClearConfirm(e.target.value)}
        />
        <div className="mt-4 flex justify-end gap-2">
          <Button variant="ghost" onClick={() => setClearOpen(false)}>
            取消
          </Button>
          <Button disabled={clearing} onClick={handleClearIndex}>
            {clearing ? "清空中…" : "确认清空"}
          </Button>
        </div>
      </GlassModal>
    </div>
  );
}
