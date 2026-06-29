# Disk Helper v1.1 实现计划

> **状态**: 进行中（2026-06-23 定稿）→ 基线 tag `v1.0.0` · PRD 已确认见 `docs/current/modules/disk-helper/v1.1/`
>
> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 v1.0 基线上交付 Disk Helper v1.1——**AI 多轮自然对话 + 路径来源推断 + 可选深度分析/联网检索（本地 Ollama 可用）**，以及 **扫描降内存 + 进度准确 + 各页加载加速**。

**Architecture:** 延续 Tauri 2 + React + Rust + SQLite。v1.1 增量集中在 `services/ai/`（编排、推断、搜索、工具）、`services/scan/`（性能档位）、`services/rules/cleanup`（缓存失效）、`AnalysisPage` 与 `SettingsPage`。AI 编排层在 Rust 侧完成：路径推断 → 可选本机工具 → 可选联网检索 → 拼装 prompt → Ollama/DeepSeek；**文件内容与完整路径不上云、不发搜索引擎**。

**Tech Stack:** 与 v1 相同 + v1.1 新增 `reqwest` 搜索客户端（DuckDuckGo HTML Lite 或 Brave Search API 二选一，默认 DuckDuckGo 免 Key）· 可选 MCP 客户端（后期）

**关联文档:**

- [产品概要说明书_v1.1.md](../current/modules/disk-helper/v1.1/产品概要说明书_v1.1.md)
- [PRD_AI智能分析_v1.1.md](../current/modules/disk-helper/v1.1/PRD_AI智能分析_v1.1.md)
- [PRD_性能与体验_v1.1.md](../current/modules/disk-helper/v1.1/PRD_性能与体验_v1.1.md)
- [详细设计_v1.md](../current/modules/disk-helper/详细设计_v1.md)（IPC/表结构基线）

---

## 文件结构（v1.1 增量）

```text
disk-helper/src-tauri/src/
├── services/
│   ├── ai/
│   │   ├── service.rs              # 编排：多轮、推断、搜索、prompt
│   │   ├── path_inference.rs       # NEW：路径模式知识库 + 置信度
│   │   ├── web_search.rs           # NEW：泛化检索词 + HTTPS 搜索 + 摘要
│   │   ├── local_tools.rs          # NEW：深度分析只读元数据（可选 MCP 适配）
│   │   ├── cloud_deepseek.rs
│   │   └── local_ollama.rs
│   ├── config.rs                   # + ai_deep_analysis_enabled, ai_web_search_enabled, scan_performance_profile
│   ├── scan/
│   │   ├── engine.rs               # 批次/并行度按档位；内存峰值优化
│   │   └── profiles.rs             # NEW：balanced | low_impact
│   └── rules.rs                    # 缓存代际 + scan 完成失效
├── commands/
│   └── ai.rs                       # ai_chat 扩展响应字段
└── data/
    └── path_patterns.v1.1.json     # NEW：≥20 条推断规则（Chrome/Temp/HF/…）

disk-helper/src/
├── pages/analysis/AnalysisPage.tsx # 多轮 UI、推断块、引用块、状态条
├── pages/settings/SettingsPage.tsx # 深度分析/联网检索/扫描档位
├── lib/api.ts                      # aiChat 传 history；新 settings 字段
└── types/index.ts                  # InferenceBlock, WebRefBlock, ChatMessage 扩展

docs/current/modules/disk-helper/
└── v1.1-acceptance-checklist.md    # M11 产出
```

---

## 里程碑总览

| 阶段 | 状态 | 名称 | 产出 | 用户可感知 |
| --- | --- | --- | --- | --- |
| M7 | ⏳ | AI 后端 | 推断引擎、多轮 chat、联网检索、深度分析工具 | IPC 返回推断/引用 |
| M8 | ⏳ | AI 前端 | 分析页多轮、设置开关、首次确认 | AI 页自然对话 |
| M9 | ⏳ | 性能后端 | 扫描档位、内存批次、清理缓存失效 | 扫描更省资源 |
| M10 | ⏳ | 性能前端 | 总览进度、各页 loading 统一 | 少卡顿 |
| M11 | ⏳ | v1.1 验收 | 验收清单、文档、tag v1.1.0 | 可发布 v1.1 |

### 当前进度（2026-06-23）

- **PRD** ✅ — v1.1 产品概要 + 菜单 PRD 已确认
- **Plan** ✅ — 本文档
- **M7–M11** ⏳ — 待开发

**开发分支建议:** `feature/v1.1` from tag `v1.0.0`

**本地验证（全程）:**

```bash
cd disk-helper
cargo test                          # Rust 单元/集成
npm run build
npm run tauri dev                   # VITE_USE_MOCK=false
# 小盘扫描：
# $env:SCAN_ROOT="src-tauri/tests/fixtures/sample_tree"
```

---

## M7：AI 后端

### Task 7.1：设置字段与迁移

**Files:**

- Modify: `disk-helper/src-tauri/src/services/config.rs`
- Modify: `disk-helper/src/types/index.ts`（前端 AppSettings）
- Modify: `disk-helper/src/pages/settings/SettingsPage.tsx`

- [ ] **Step 1:** `AppSettings` 增加：
  - `ai_deep_analysis_enabled: bool`（默认 `false`）
  - `ai_web_search_enabled: bool`（默认 `false`）
  - `scan_performance_profile: ScanPerformanceProfile`（`balanced` | `low_impact`，默认 `balanced`）
- [ ] **Step 2:** `ConfigSaveInput` 同步 camelCase 可选字段；`config_get`/`config_save` 持久化 JSON
- [ ] **Step 3:** `settings_change` 审计：深度分析/联网检索/扫描档位变更
- [ ] **Step 4:** 前端设置 Tab 增加三个控件 + 联网检索首次开启二次确认文案

**验证:** 修改设置 → 重启 → 值仍生效。

---

### Task 7.2：路径推断引擎

**Files:**

- Create: `disk-helper/src-tauri/data/path_patterns.v1.1.json`
- Create: `disk-helper/src-tauri/src/services/ai/path_inference.rs`
- Modify: `disk-helper/src-tauri/src/services/ai/mod.rs`

- [ ] **Step 1:** JSON 规则结构：`pattern`（glob/contains）、`source_label`、`description`、`confidence_default`（high/medium/low）
- [ ] **Step 2:** 内置 ≥20 条：Chrome Cache、Windows Temp、npm cache、`.cache/huggingface`、`AppData\Local\Packages`、Gradle、Conda 等
- [ ] **Step 3:** `infer_path(path, is_dir, ext) -> PathInference { label, summary, confidence, evidence[] }`
- [ ] **Step 4:** 单元测试：HF 路径 → medium/high；未知路径 → low

**验证:** `cargo test path_inference` 全绿。

---

### Task 7.3：联网检索服务

**Files:**

- Create: `disk-helper/src-tauri/src/services/ai/web_search.rs`
- Modify: `disk-helper/src-tauri/src/services/ai/service.rs`

- [ ] **Step 1:** `build_search_query(question, inferences) -> String` — 泛化，剥离 `C:\Users\...`，用 `{user}` 占位
- [ ] **Step 2:** `should_search(settings, inferences, question) -> bool` — 联网开关开 **且**（置信度 low **或** 问题含关键词「是什么/官方/能否删/huggingface/…」）
- [ ] **Step 3:** 实现 `search_public(query) -> Vec<SearchSnippet { title, url, snippet }>` — **默认 DuckDuckGo HTML Lite**（免 API Key）；预留 Brave API Key 配置项（可选 Task，不阻塞 M7）
- [ ] **Step 4:** 会话内计数，超过 5 次返回 `AiWebSearchLimit` 错误码
- [ ] **Step 5:** 无网络 / HTTP 失败 → 空摘要 + 降级，不阻断 chat

**验证:** 集成测试 mock HTTP；真实网络下 query `"huggingface hub cache delete safe"` 返回非空 snippets。

---

### Task 7.4：深度分析本地工具

**Files:**

- Create: `disk-helper/src-tauri/src/services/ai/local_tools.rs`

- [ ] **Step 1:** `read_metadata(path) -> FileMetadataHint` — 大小、mtime、扩展名、Windows 下可选只读 peek 前 512 字节（hex/可打印字符），**不上传**
- [ ] **Step 2:** 仅在 `ai_deep_analysis_enabled` 时调用；超时 3s 降级
- [ ] **Step 3:** （可选）MCP 适配 trait `LocalToolProvider`，默认 `BuiltinToolProvider`

**验证:** 对 fixture 文件返回 mtime；关闭开关时不调用。

---

### Task 7.5：AI 编排与 IPC 扩展

**Files:**

- Modify: `disk-helper/src-tauri/src/services/ai/service.rs`
- Modify: `disk-helper/src-tauri/src/commands/ai.rs`
- Modify: `disk-helper/src/lib/api.ts`

- [ ] **Step 1:** 移除 `build_system_prompt` 中强制四段结构；改为自然语言 + 安全约束 + 规则优先
- [ ] **Step 2:** `ai_chat` 入参增加 `history: Vec<ChatTurn { role, content }>`（最多 10 轮）；出参增加：
  - `inferences: Vec<PathInferenceDto>`（按 context 路径）
  - `web_refs: Vec<WebRefDto>`（可选）
  - `disclaimer: String`
- [ ] **Step 3:** 编排顺序：推断 → 深度工具 → 联网检索 → 组装 messages → `dispatch_chat`
- [ ] **Step 4:** 云端模式：发送至 DeepSeek 的仍仅为脱敏 context + 摘要 + history；**检索摘要本地获取后一并发送**
- [ ] **Step 5:** 本地 Ollama：全部 prompt 在本机构建，仅搜索步骤走 HTTPS
- [ ] **Step 6:** `audit ai_query` 增加 flags：`deep_analysis`, `web_search`（不记 query 全文）

**验证:** `cargo test` + 手动 IPC：HF 路径 + 联网开 → 响应含 `web_refs`。

---

## M8：AI 前端

### Task 8.1：分析页多轮与结构化块

**Files:**

- Modify: `disk-helper/src/pages/analysis/AnalysisPage.tsx`
- Modify: `disk-helper/src/types/index.ts`
- Modify: `disk-helper/src/lib/api.ts`

- [ ] **Step 1:** 发送时附带 `messages` 最近 10 轮（role + content）
- [ ] **Step 2:** 左栏上下文项展示 `推断预览`（IPC 返回或客户端预推断）
- [ ] **Step 3:** AI 消息渲染：`来源推断` 标签块（置信度颜色）、`参考来源` 折叠块（title + 外链）
- [ ] **Step 4:** 页头状态：`深度分析：开/关` · `联网检索：开/关`（只读，链到设置）
- [ ] **Step 5:** 「新对话」清空 messages，可选保留 context
- [ ] **Step 6:** loading / 停止生成（若后端支持 cancel 可 Phase 2，否则仅 loading）

**验证:** 连续 3 轮追问语义连贯；HF 样例见 AC-04b。

---

### Task 8.2：设置页 AI 增强区

**Files:**

- Modify: `disk-helper/src/pages/settings/SettingsPage.tsx`

- [ ] **Step 1:** AI 分区：深度分析 Switch + 说明
- [ ] **Step 2:** 联网检索 Switch + **首次开启 Modal**（检索词发往搜索服务、不含完整路径）
- [ ] **Step 3:** 扫描 Tab：性能档位单选（平衡 / 低占用）

**验证:** 开关持久化；Modal 仅首次展示。

---

## M9：性能后端

### Task 9.1：扫描性能档位

**Files:**

- Create: `disk-helper/src-tauri/src/services/scan/profiles.rs`
- Modify: `disk-helper/src-tauri/src/services/scan/engine.rs`
- Modify: `disk-helper/src-tauri/src/services/scan/mod.rs`

- [ ] **Step 1:** `ScanProfile { batch_size, walk_threads, flush_interval_ms }`：
  - `balanced`: batch 500（现状）
  - `low_impact`: batch 200, threads = max(1, cores/2), flush 间隔加长
- [ ] **Step 2:** `run_full_scan` 启动时读 `config.scan_performance_profile`
- [ ] **Step 3:** 降低峰值：batch 写入后 `Vec` clear/shrink；避免在内存中累积整批 FileRow 副本（review `engine.rs` insert 路径）
- [ ] **Step 4:** 扫描完成 emit 事件 → 清理 rules 缓存代际 +1

**验证:** 同 fixture 树对比 v1.0 峰值内存（任务管理器）；low_impact 线程数符合 PRD。

---

### Task 9.2：清理建议缓存与 SQL 优化

**Files:**

- Modify: `disk-helper/src-tauri/src/services/rules.rs`
- Modify: `disk-helper/src-tauri/src/services/cleanup.rs`

- [ ] **Step 1:** 缓存 key = `(index_generation, rule_version)`；scan 完成 / index_clear 递增 generation
- [ ] **Step 2:** 确认路径前缀 SQL 过滤仍生效；分页 API `offset/limit` 默认 50
- [ ] **Step 3:** 添加 debug 级耗时日志 `cleanup_suggest_ms`

**验证:** 二次打开 cleanup IPC P95 ≤2s（fixture 或 dev 盘）。

---

### Task 9.3：进度与快照

**Files:**

- Modify: `disk-helper/src-tauri/src/services/scan/engine.rs`
- Modify: `disk-helper/src-tauri/src/commands/scan.rs`

- [ ] **Step 1:** 进度 emit 同时带 `scanned_files` + `percent`；percent 分母 = `last_completed_scanned_files` 或保守估计
- [ ] **Step 2:** `scan_get_snapshot` 文档化字段；前端总览展示「已扫描 N 个文件」

**验证:** 第二次全量扫描进度与文件数偏差 ≤10%（AC-06）。

---

## M10：性能前端

### Task 10.1：总览扫描 UX

**Files:**

- Modify: `disk-helper/src/pages/overview/OverviewPage.tsx`

- [ ] **Step 1:** 扫描条展示文件数 + 百分比 + 暂停/取消（已有则 polish）
- [ ] **Step 2:** 扫描中切页不阻塞（TanStack Query staleTime / 非 blocking invoke）

**验证:** 扫描中打开清理页 ≤5s 有 loading 或内容（AC-05）。

---

### Task 10.2：列表页 loading 统一

**Files:**

- Modify: `disk-helper/src/pages/cleanup/CleanupPage.tsx`
- Modify: `disk-helper/src/pages/quarantine/QuarantinePage.tsx`
- Modify: `disk-helper/src/pages/explorer/ExplorerPage.tsx`（Top 列表）

- [ ] **Step 1:** 首屏 `isLoading` 展示统一 Spinner +「加载中…」
- [ ] **Step 2:** 分页默认 50；隔离区/清理 debounce 筛选（清理页已有则确认）
- [ ] **Step 3:** `staleTime` 30s 隔离区缓存

**验证:** AC-07 清理 2s；AC 列表 1s。

---

## M11：v1.1 验收与发布

### Task 11.1：验收清单

**Files:**

- Create: `docs/current/modules/disk-helper/v1.1-acceptance-checklist.md`

- [ ] **Step 1:** 从产品概要 AC-01～AC-13 + AC-04b 复制为可勾选清单
- [ ] **Step 2:** 记录验收机器配置（CPU/内存/盘符）
- [ ] **Step 3:** 路径推断用例 ≥20 条测试结果表

---

### Task 11.2：文档与 tag

**Files:**

- Create: `docs/current/modules/disk-helper/RELEASE_v1.1.md`
- Modify: `docs/current/Index.md`、`docs/superpowers/Index.md`
- Modify: 本 plan 里程碑状态 → 已完成

- [ ] **Step 1:** RELEASE_v1.1.md（相对 v1.0 变更摘要）
- [ ] **Step 2:** Git tag `v1.1.0`（用户确认后）
- [ ] **Step 3:** `tauri build` 安装包 smoke test

---

## 错误码（v1.1 新增）

| 码 | 含义 | 用户提示 |
| --- | --- | --- |
| `AiWebSearchLimit` | 单会话搜索超 5 次 | 本轮联网检索次数已达上限 |
| `AiWebSearchUnavailable` | 无网或搜索失败 | 未能联网检索，将使用本地知识回答 |
| `AiDeepAnalysisTimeout` | 本地工具超时 | 深度分析超时，已使用基础元数据 |

---

## 测试策略

| 层级 | 范围 |
| --- | --- |
| Rust 单元 | `path_inference`、`build_search_query` 脱敏、profile 参数 |
| Rust 集成 | `ai_chat` mock search HTTP；scan profile batch |
| 前端 | `npm run build`；AnalysisPage 手工多轮 |
| 性能 | 同机 v1.0.0 vs v1.1 内存峰值；cleanup 二次打开计时 |
| 安全 | 断言 search query 不含 `\Users\真实用户名` |

---

## Commit 建议（与 v1 一致）

| Commit | 范围 |
| --- | --- |
| `feat(disk-helper-be): v1.1 AI backend (inference, web search, multi-turn)` | M7 |
| `feat(disk-helper-fe): v1.1 AI analysis UI and settings` | M8 |
| `feat(disk-helper-be): v1.1 scan profiles and cleanup cache` | M9 |
| `feat(disk-helper-fe): v1.1 performance UX polish` | M10 |
| `docs(disk-helper): v1.1 acceptance and release` | M11 |

---

## 风险与裁剪

| 风险 | 缓解 |
| --- | --- |
| DuckDuckGo HTML 结构变更 | 抽象 `SearchProvider` trait；备选 Brave API |
| 小模型多轮质量差 | 限制 history 10 轮；prompt 注入推断+检索摘要 |
| 内存优化不达标 | 优先 low_impact 档位 + 减小 batch；必要时 streaming insert |
| MCP 工期 | v1.1 以 `local_tools.rs` 内置为准；MCP 作 v1.2 |

**可裁剪（若工期紧）:** Brave API 集成、流式取消生成、MCP 适配 — **不可裁剪:** 路径推断、多轮、联网检索（DuckDuckGo）、扫描 low_impact 档位、清理缓存。

---

## 验收对照（快速索引）

| AC | 里程碑 |
| --- | --- |
| AC-01 多轮 | M7 + M8 |
| AC-02 路径推断 | M7 |
| AC-04b 本地+联网 HF | M7 + M8 |
| AC-05/06 扫描 | M9 + M10 |
| AC-07 清理加载 | M9 + M10 |
| AC-11 内存 -30% | M9 |

完整定义见 [产品概要说明书_v1.1.md](../current/modules/disk-helper/v1.1/产品概要说明书_v1.1.md) §六。
