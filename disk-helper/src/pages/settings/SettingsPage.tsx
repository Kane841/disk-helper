import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { useTheme } from "next-themes";
import { api } from "@/lib/api";
import { useToastStore } from "@/stores/app-store";
import type { AppSettings } from "@/types";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";
import { PageHeader } from "@/components/PageHeader";
import { Button, GlassInput } from "@/components/ui/button";
import { Card, CardBody } from "@/components/ui/card";

type Tab = "ai" | "appearance" | "quarantine" | "scan" | "advanced";

export function SettingsPage() {
  const { theme, setTheme } = useTheme();
  const queryClient = useQueryClient();
  const showToast = useToastStore((s) => s.show);
  const [tab, setTab] = useState<Tab>("ai");
  const [draft, setDraft] = useState<Partial<AppSettings>>({});
  const [apiKey, setApiKey] = useState("");
  const [testResult, setTestResult] = useState<string | null>(null);

  const { data: settings } = useQuery({
    queryKey: ["settings"],
    queryFn: () => api.configGet(),
  });

  const merged = { ...settings, ...draft } as AppSettings | undefined;

  const update = (partial: Partial<AppSettings>) => {
    setDraft((d) => ({ ...d, ...partial }));
  };

  const save = async () => {
    await api.configSave({
      ...draft,
      has_api_key: apiKey.length > 0 ? true : settings?.has_api_key,
    });
    setDraft({});
    queryClient.invalidateQueries({ queryKey: ["settings"] });
    showToast("设置已保存（模拟）");
  };

  const testConnection = async () => {
    const res = await api.aiTestConnection();
    setTestResult(res.message);
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
                      onClick={() => setTheme(t)}
                      className={cn(
                        "rounded-xl px-4 py-2 text-sm capitalize",
                        theme === t ? glass.navActive : glass.navIdle,
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
            )}
            <div className="flex gap-2 border-t border-white/25 pt-4 dark:border-white/10">
              <Button onClick={save}>保存</Button>
              <Button variant="secondary" onClick={() => setDraft({})}>
                重置未保存更改
              </Button>
            </div>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}
