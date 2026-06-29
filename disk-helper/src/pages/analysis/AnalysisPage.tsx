import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useSelectionStore, useToastStore } from "@/stores/app-store";
import { api } from "@/lib/api";
import { TauriApiError } from "@/lib/tauri-client";
import type { ChatMessage } from "@/types";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/cn";
import { glass } from "@/lib/glass";
import { text } from "@/lib/theme";
import { PageHeader } from "@/components/PageHeader";
import { RiskBadge } from "@/components/RiskBadge";
import { Button, GlassInput } from "@/components/ui/button";

export function AnalysisPage() {
  const { items, source, clear } = useSelectionStore();
  const showToast = useToastStore((s) => s.show);
  const { data: settings } = useQuery({
    queryKey: ["settings"],
    queryFn: () => api.configGet(),
  });
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);

  const send = async (text?: string) => {
    const question = (text ?? input).trim();
    if (!question) {
      showToast("请输入问题");
      return;
    }
    const userMsg: ChatMessage = {
      id: `u-${Date.now()}`,
      role: "user",
      content: question,
      sent_at: new Date().toISOString(),
    };
    setMessages((m) => [...m, userMsg]);
    setInput("");
    setLoading(true);
    try {
      const res = await api.aiChat(question, items);
      setMessages((m) => [
        ...m,
        {
          id: `a-${Date.now()}`,
          role: "assistant",
          content: res.message + "\n\n---\n*" + res.disclaimer + "*",
          sent_at: new Date().toISOString(),
        },
      ]);
    } catch (error) {
      const message =
        error instanceof TauriApiError ? error.message : "AI 请求失败，请稍后重试";
      showToast(message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-full">
      <aside className={cn(glass.panel, "m-3 mr-0 w-72 shrink-0 overflow-auto p-4")}>
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold">当前上下文</h3>
          <Button variant="ghost" size="sm" onClick={clear}>
            清空
          </Button>
        </div>
        <p className={cn("mt-2 text-xs", text.muted)}>
          来源：{source === "none" ? "无" : source === "browse" ? "空间浏览" : "安全清理"}
        </p>
        <ul className="mt-4 space-y-2">
          {items.length === 0 ? (
            <li className={cn("text-xs", text.faint)}>从浏览或清理页带入上下文</li>
          ) : (
            items.map((item) => (
              <li
                key={item.path}
                className={cn(glass.panel, "p-3 text-xs")}
              >
                <p className="truncate font-medium">{item.path}</p>
                <p className={cn("mt-1", text.muted)}>{formatBytes(item.size_bytes)}</p>
                {item.risk && (
                  <div className="mt-2">
                    <RiskBadge risk={item.risk} />
                  </div>
                )}
              </li>
            ))
          )}
        </ul>
      </aside>
      <div className="flex min-w-0 flex-1 flex-col">
        <PageHeader
          title="AI 智能分析"
          description={
            settings
              ? settings.ai_mode === "local"
                ? `本地 Ollama · ${settings.ollama_model}`
                : settings.has_api_key
                  ? "云端 DeepSeek API"
                  : "云端模式：请先在设置中配置 API Key"
              : "云端 DeepSeek / 本地 Ollama"
          }
        />
        <div className="flex-1 overflow-auto p-6">
          <div className="mx-auto max-w-2xl space-y-4">
            {messages.length === 0 && !loading && (
              <p className={cn("text-center text-sm", text.muted)}>
                输入问题，或从浏览/清理页带入上下文后提问
              </p>
            )}
            {messages.map((msg) => (
              <div
                key={msg.id}
                className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
              >
                <div
                  className={cn(
                    "max-w-[90%] rounded-2xl px-4 py-3 text-sm whitespace-pre-wrap backdrop-blur-xl",
                    msg.role === "user"
                      ? "glass-panel-strong bg-emerald-500/15 text-emerald-950 ring-emerald-500/20 dark:text-emerald-100"
                      : cn(glass.panel, text.body),
                  )}
                >
                  {msg.content}
                </div>
              </div>
            ))}
            {loading && (
              <p className={cn("text-center text-sm", text.muted)}>AI 思考中...</p>
            )}
          </div>
        </div>
        <div className={cn("glass-header m-3 mt-0 rounded-2xl p-4")}>
          <div className="mx-auto flex max-w-2xl flex-wrap gap-2 pb-3">
            <Button
              variant="secondary"
              size="sm"
              disabled={items.length === 0}
              onClick={() => send("这些文件/文件夹可以删除吗？有什么风险？")}
            >
              能删吗
            </Button>
            <Button
              variant="secondary"
              size="sm"
              disabled={items.length === 0}
              onClick={() => send("如果删除了，应该如何恢复？")}
            >
              怎么恢复
            </Button>
          </div>
          <div className="mx-auto flex max-w-2xl gap-2">
            <GlassInput
              className="min-w-0 flex-1"
              placeholder="输入问题..."
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && !loading && send()}
              disabled={loading}
            />
            <Button onClick={() => send()} disabled={loading}>
              发送
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
