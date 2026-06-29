# Disk Helper v1 实现计划

> **状态**: 已完成并归档（2026-06-23）→ 验收见 `docs/current/modules/disk-helper/v1-acceptance-checklist.md` · 发布见 `RELEASE_v1.md` · Git tag `v1.0.0`
>
> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 交付 Windows 本地桌面版 Disk Helper v1——C 盘扫描浏览、规则安全清理、隔离区、操作日志，以及**云端 DeepSeek API + 本地 Ollama（DeepSeek 小模型）**双通道 AI 分析。

**Architecture:** Tauri 2 壳 + React 前端 + Rust 领域服务 + SQLite 索引。前端经 Tauri Command 访问后端；AI 调用仅在 Rust 侧 outbound。v1 增量扫描采用 mtime/size 浅层 diff 方案（已确认接受）。

**Tech Stack:** Tauri 2 · React 18 · Vite · TypeScript · Tailwind · shadcn/ui · TanStack Query · Zustand · Rust · rusqlite · jwalk · rayon · globset · reqwest · keyring · Ollama（本地）

**关联文档:** `docs/current/modules/disk-helper/产品概要说明书_v1.md` · `详细设计_v1.md` · `PRD_*.md`

---

## 文件结构（目标仓库）

```text
disk-helper/                          # 项目根（当前仓库根目录下创建）
├── package.json
├── vite.config.ts
├── tailwind.config.ts
├── src/
│   ├── app/                          # AppShell、Router、ThemeProvider
│   ├── pages/                        # 7 个业务页 + 原型阶段先完成
│   ├── components/                   # Treemap、RiskBadge、CapacityCard…
│   ├── lib/tauri-client.ts           # invoke 封装
│   ├── mocks/                        # 原型阶段 Mock 数据与 fake API
│   └── stores/selection-context.ts
├── src-tauri/
│   ├── src/
│   │   ├── commands/                 # 每个 domain 一个 mod
│   │   ├── services/                 # scan/index/rules/cleanup/…
│   │   ├── db/migrations/
│   │   └── error.rs
│   ├── rules/builtin-rules.v1.json
│   └── tauri.conf.json
└── docs/current/modules/disk-helper/
```

---

## 里程碑总览


| 阶段  | 状态  | 名称                    | 产出                                   | 用户可感知        |
| --- | --- | --------------------- | ------------------------------------ | ------------ |
| M0  | ✅   | 环境就绪                  | Rust/Node/Tauri toolchain            | —            |
| M1  | ✅   | **UI 原型（Checkpoint）** | 7 页 + Mock 数据，浏览器可跑                  | **可体验全流程界面** |
| M2  | ✅   | Tauri 壳 + IPC 骨架      | 桌面窗口 + `{ data, error }` 包络 + SQLite | 桌面版 + 真实 IPC |
| M3  | ✅   | 扫描与索引                 | 全量/增量扫描 + SQLite 索引 + Explorer 真数据   | 总览/浏览有真数据    |
| M4  | ✅   | 清理闭环                  | 规则 + 隔离区 + 日志                        | 可安全软删除       |
| M5  | ✅   | 双 AI                  | 云端 DeepSeek + Ollama 本地              | AI 页可用       |
| M6  | ✅   | 收尾验收                  | 设置、导出、Bugfix                         | v1 自用可交付     |


### 当前进度（截至 2026-06-23）

- **M1** ✅ — UI 原型与暗色主题修复（commit `2ea78b6`）
- **M2** ✅ — `ApiResponse` 包络、`AppState` + SQLite 六表迁移、`volume_get_c_drive` stub → 真实 IPC 切换
- **M3** ✅ — C 盘容量、全量扫描（jwalk + 事件）、IndexService 聚合查询、增量扫描（depth≤2 浅层 diff）
- **M4** ✅ — 内置规则引擎（globset + min_age_days）、隔离区移入/还原/永久删除、cleanup_execute + 审计日志
- **M5** ✅ — AiProvider 双通道（CloudDeepSeek + LocalOllama）、config_get/save + keyring、ai_chat / ai_test_connection、Analysis/Settings 接 IPC
- **M6** ✅ — 设置全量持久化、index_clear、跨页上下文 session 持久、ErrorBoundary、空状态、v1 验收清单
- **v1 验收** ✅ — AC-01～AC-13 及发布前检查已通过（见 `v1-acceptance-checklist.md`，2026-06-23）
- **v1 发布** ✅ — Git tag `v1.0.0` · 文档见 `RELEASE_v1.md`
- **下一步** — 启动 v1.1+ 新功能：新建 plan/spec，基于 v1 代码基线迭代

**本地验证：**

```bash
cd disk-helper
npm run tauri dev          # 桌面版，VITE_USE_MOCK=false
# 开发扫描可缩小范围：
# $env:SCAN_ROOT="src-tauri/tests/fixtures/sample_tree"
cargo test                 # src-tauri，15 项通过（含规则/隔离区/扫描/索引）
```

---

## M0：环境与仓库初始化

> **进度（2026-06-22）**
>
> - [x] Node.js v24 已就绪
> - [x] `disk-helper/` Tauri 2 + React + TS 脚手架已创建
> - [x] npm 依赖已安装（router / query / zustand / tailwind）
> - [x] `scripts/setup-dev-env.ps1` + `dev-environment.md`
> - [x] Rust stable + Cargo（rsproxy 镜像）
> - [x] `.cargo/config.toml` crates 镜像
> - [x] `npm run tauri dev` 首次验证通过
> - [ ] Ollama + deepseek-r1:1.5b（M5 前，可选提前装）

### Task 0.1：开发环境检查

**Files:** 无（本机环境）

- [ ] **Step 1:** 确认已安装 Node.js ≥ 20、Rust stable、Visual Studio Build Tools（Windows C++）
- [ ] **Step 2:** 安装 Tauri CLI：`cargo install tauri-cli --version "^2"`
- [ ] **Step 3:** 确认 Ollama 已安装（AI 阶段需要）：`ollama --version`
- [ ] **Step 4:** 预拉取本地模型：`ollama pull deepseek-r1:1.5b`（备选 `deepseek-r1:7b` 配置项可改）

**验证:** 上述命令均 exit 0。

---

### Task 0.2：初始化 Tauri + React 项目

**Files:**

- Create: `disk-helper/` 下全部脚手架（`npm create tauri-app@latest` 选型 React + TypeScript）

- [ ] **Step 1:** 在仓库根目录执行：

```bash
cd c:\WorkPlace\Java\Disk_helper
npm create tauri-app@latest disk-helper -- --template react-ts
```

- [ ] **Step 2:** 进入目录安装 UI 依赖：

```bash
cd disk-helper
npm install react-router-dom @tanstack/react-query zustand next-themes
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

- [ ] **Step 3:** 初始化 shadcn/ui（Vite 模式）并安装基础组件：`button`, `card`, `table`, `dialog`, `input`, `tabs`, `progress`, `badge`, `toast`

- [ ] **Step 4:** 配置 `vite.config.ts` alias `@/` → `src/`

- [ ] **Step 5:** 首次运行验证：

```bash
npm run tauri dev
```

**Expected:** 空壳窗口正常启动。

- [ ] **Step 6:** Commit

```bash
git add disk-helper/
git commit -m "chore: scaffold Tauri 2 + React + TS project"
```

---

## M1：UI 原型（★ 用户 Checkpoint）

> **进度（2026-06-22）：已完成** — 7 页 + Mock API + 路由/主题/交互
> **体验：** `cd disk-helper && npm run dev` → [http://localhost:1420](http://localhost:1420)  
> **指南：** `prototype-walkthrough.md`

> **目的：** 在接入 Rust 扫盘之前，用 **Mock 数据** 完成全部 7 个页面的布局、导航、交互态，供产品体验与视觉确认。  
> **运行方式：** 仅前端 `npm run dev`（浏览器），不依赖 Tauri/Rust。  
> **完成标准：** 用户逐页点击验收，确认布局/信息架构/交互流程后再进入 M2。

### Task 1.1：应用壳与路由

**Files:**

- Create: `src/app/AppShell.tsx`
- Create: `src/app/router.tsx`
- Create: `src/app/theme-provider.tsx`
- Modify: `src/main.tsx`, `src/App.tsx`

- [ ] **Step 1:** 实现 `AppShell`——左侧或顶部主导航 5 项：总览 / 浏览 / 清理 / 分析 / 设置；设置内含 Tab 跳转操作日志
- [ ] **Step 2:** 配置路由：

```text
/overview
/explorer
/cleanup
/quarantine
/analysis
/settings
/settings/audit
```

- [ ] **Step 3:** 接入 `next-themes`，支持 system/light/dark 切换（设置页先放占位开关）
- [ ] **Step 4:** 运行 `npm run dev`，确认 7 路由可切换、无 404

---

### Task 1.2：Mock 数据层

**Files:**

- Create: `src/mocks/fixtures.ts`
- Create: `src/mocks/mock-api.ts`
- Create: `src/lib/api.ts`（原型阶段指向 mock-api）

- [ ] **Step 1:** 在 `fixtures.ts` 定义静态数据（贴近真实 C 盘场景）：

```typescript
// 示例结构（实现时补全）
export const mockVolume = {
  drive: "C:",
  total_bytes: 512_000_000_000,
  used_bytes: 420_000_000_000,
  free_bytes: 92_000_000_000,
  usage_percent: 82.0,
};

export const mockCategories = [
  { code: "system", size_bytes: 80_000_000_000, ratio: 0.19 },
  { code: "cache_temp", size_bytes: 45_000_000_000, ratio: 0.11 },
  // …共 6 类
];

export const mockFileTree = [ /* C:\ 下 3～4 层样例节点 */ ];
export const mockTopFiles = [ /* 10 条大文件 */ ];
export const mockSuggestions = [ /* 8 条清理建议，含 safe/caution/danger */ ];
export const mockQuarantineItems = [ /* 3 条隔离项 */ ];
export const mockAuditLogs = [ /* 10 条日志 */ ];
export const mockAiMessages = [ /* 2 轮示例对话 */ ];
```

- [ ] **Step 2:** `mock-api.ts` 用 `Promise` + `setTimeout(200ms)` 模拟异步，暴露与详细设计 Command 同名的 TS 函数（如 `volumeGetCDrive()`）
- [ ] **Step 3:** `api.ts` 导出：`export const api = import.meta.env.VITE_USE_MOCK !== 'false' ? mockApi : tauriApi`（原型期默认 mock）

---

### Task 1.3：页面原型 — 磁盘总览

**Files:**

- Create: `src/pages/overview/OverviewPage.tsx`
- Create: `src/components/CapacityCard.tsx`
- Create: `src/components/CategoryGrid.tsx`
- Create: `src/components/ScanStatusBar.tsx`

- [ ] **Step 1:** 容量卡片 + 进度条 + 告警条（mock free_bytes 触发 warning）
- [ ] **Step 2:** 六类占用网格，点击跳转 `/explorer`（带 query 可选）
- [ ] **Step 3:** 扫描状态条 + 按钮：开始/暂停/继续/取消——用本地 `useState` 模拟进度 0→100%
- [ ] **Step 4:** 快捷入口按钮跳转浏览/清理/分析

**验收:** 总览信息完整、扫描动画流畅、跳转正确。

---

### Task 1.4：页面原型 — 空间浏览

**Files:**

- Create: `src/pages/explorer/ExplorerPage.tsx`
- Create: `src/components/FileTree.tsx`
- Create: `src/components/DiskTreemap.tsx`（可用 `@nivo/treemap` 或简化 div 方块）
- Create: `src/components/TopFilesTable.tsx`

- [ ] **Step 1:** 左树 + 右 Treemap 联动（mock 数据）
- [ ] **Step 2:** 工具栏：大文件 Top / 大文件夹 Top / 搜索框
- [ ] **Step 3:** 底部详情条：路径、大小、风险 Badge
- [ ] **Step 4:** 「问 AI」「查看清理建议」按钮写入 Zustand `selectionContext` 并跳转

**验收:** 树选中 ↔ Treemap 高亮 ↔ 详情条一致。

---

### Task 1.5：页面原型 — 安全清理 + 隔离区

**Files:**

- Create: `src/pages/cleanup/CleanupPage.tsx`
- Create: `src/pages/quarantine/QuarantinePage.tsx`
- Create: `src/components/RiskBadge.tsx`
- Create: `src/components/CleanupConfirmDialog.tsx`

- [ ] **Step 1:** 清理页：筛选 + 可勾选表格 + 预计释放空间汇总
- [ ] **Step 2:** 危险项默认 disabled；勾选「我了解风险」+ 确认文案后可勾选
- [ ] **Step 3:** 确认对话框 → toast「已移入隔离区（模拟）」
- [ ] **Step 4:** 隔离区页：列表 + 还原/永久删除对话框（模拟状态变更）

**验收:** 风险三色清晰；危险项保护交互符合 PRD。

---

### Task 1.6：页面原型 — AI 分析 + 设置 + 操作日志

**Files:**

- Create: `src/pages/analysis/AnalysisPage.tsx`
- Create: `src/pages/settings/SettingsPage.tsx`
- Create: `src/pages/audit/AuditLogPage.tsx`
- Create: `src/components/AiChatPanel.tsx`

- [ ] **Step 1:** AI 页：左上下文面板 + 右对话区；读取 `selectionContext`
- [ ] **Step 2:** Mock 发送消息后 1s 返回固定 Markdown 回复 + 免责声明
- [ ] **Step 3:** 设置页 Tab：AI 配置 / 外观 / 隔离区 / 扫描 / 高级
- [ ] **Step 4:** AI Tab 展示：**AI 模式** 单选 `本地 Ollama | 云端 DeepSeek`；Ollama 显示模型名 `deepseek-r1:1.5b`（只读默认）+ Base URL；云端显示 API Key 密码框
- [ ] **Step 5:** 操作日志页：筛选 + 表格 + 导出按钮（toast 模拟）

**验收:** AI 模式切换 UI 可见；设置各 Tab 字段与 PRD 一致。

---

### Task 1.7：原型交付与 Checkpoint

**Files:**

- Create: `docs/current/modules/disk-helper/prototype-walkthrough.md`（可选，一页体验说明）

- [ ] **Step 1:** 在 `package.json` 增加脚本：`"dev:prototype": "vite"`（若与 tauri 共存已满足则跳过）
- [ ] **Step 2:** 运行并自测全流程：

```bash
cd disk-helper
npm run dev
# 浏览器打开 http://localhost:5173
# 路径：总览 → 扫描(模拟) → 浏览 → 选目录 → AI → 清理 → 隔离区 → 设置 → 日志
```

- [ ] **Step 3:** 截图或录屏备用（可选）
- [ ] **Step 4:** **STOP — 用户 Checkpoint**

> **提交给用户：** 「UI 原型已就绪，请运行 `npm run dev` 体验。请反馈：导航/布局/信息密度/配色/缺啥字段。确认后再继续 M2。」

- [ ] **Step 5:** 用户确认后 Commit

```bash
git commit -m "feat(ui): interactive prototype with mock data for all pages"
```

---

## M2：Tauri 集成与 IPC 骨架

> 前置：**M1 Checkpoint 用户已确认原型。**

### Task 2.1：Tauri Command 响应包络

**Files:**

- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1:** 定义 `AppError { code, message, target }` 与 `ApiResponse<T> { data, error }`
- [x] **Step 2:** 实现宏或 helper `ok(data)` / `err(code)`
- [x] **Step 3:** 注册 stub command `volume_get_c_drive` 返回硬编码 C 盘数据
- [x] **Step 4:** 前端 `src/lib/tauri-client.ts` 封装 `invoke`，设置 `VITE_USE_MOCK=false` 时走 Tauri

**验证:** `npm run tauri dev` 总览页显示 Rust 返回的硬编码容量。

---

### Task 2.2：AppState 与 SQLite 迁移

**Files:**

- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/migrations/001_init.sql`

- [x] **Step 1:** 按 `详细设计_v1.md` 创建 6 张表
- [x] **Step 2:** 启动时 migrations auto-run；DB 路径 `%AppData%/DiskHelper/diskhelper.db`
- [x] **Step 3:** 单元测试：迁移后表存在

```bash
cd src-tauri && cargo test db_migration -- --nocapture
```

---

## M3：扫描引擎与索引

### Task 3.1：VolumeService

**Files:**

- Create: `src-tauri/src/services/volume.rs`
- Create: `src-tauri/src/commands/volume.rs`

- [x] **Step 1:** Win32 `GetDiskFreeSpaceExW` 读 C 盘
- [x] **Step 2:** Command `volume_get_c_drive` 接真实数据
- [x] **Step 3:** 前端总览容量卡片切真实 API

---

### Task 3.2：ScanService 全量扫描

**Files:**

- Create: `src-tauri/src/services/scan.rs`
- Create: `src-tauri/src/commands/scan.rs`
- Modify: `src-tauri/Cargo.toml`（jwalk, rayon）

- [x] **Step 1:** 写 failing test：对 `tests/fixtures/sample_tree/` 扫描文件数正确

```rust
#[test]
fn full_scan_counts_files() {
    let count = scan_fixture("tests/fixtures/sample_tree");
    assert_eq!(count, 12);
}
```

- [x] **Step 2:** 实现 `jwalk` 并行 walk + batch upsert `file_entry`
- [x] **Step 3:** emit `scan://progress` 事件
- [x] **Step 4:** Commands: `scan_start`, `scan_pause`, `scan_resume`, `scan_cancel`, `scan_get_status`
- [x] **Step 5:** 前端 ScanStatusBar 接真实事件

**注意:** 首次真实 C 盘扫描耗时长，开发期可用 env `SCAN_ROOT=tests/fixtures/sample_tree` 开关。

---

### Task 3.3：IndexService 聚合与查询

**Files:**

- Create: `src-tauri/src/services/index.rs`
- Create: `src-tauri/src/commands/index.rs`

- [x] **Step 1:** 扫描完成后 `rebuild_folder_sizes()`
- [x] **Step 2:** `index_get_category_stats` SQL 聚合
- [x] **Step 3:** `index_get_children`, `index_search`, `index_get_top_files`, `index_get_top_folders`
- [x] **Step 4:** 前端 Explorer 页去掉 mock，接 TanStack Query

---

### Task 3.4：增量扫描（v1 简化）

**Files:**

- Modify: `src-tauri/src/services/scan.rs`

- [x] **Step 1:** 实现 `start_incremental_scan`：depth≤2 浅层 diff + 变更子树 re-walk
- [x] **Step 2:** tombstone 删除不存在路径
- [x] **Step 3:** 集成测试：修改 fixture 中某文件 mtime 后增量更新

---

## M4：规则引擎与安全清理

### Task 4.1：内置规则

**Files:**

- Create: `rules/builtin-rules.v1.json`
- Create: `src-tauri/src/services/rules.rs`
- Create: `src-tauri/src/commands/rules.rs`

- [x] **Step 1:** 按详细设计 4.3.3 编写规则 JSON
- [x] **Step 2:** `globset` 匹配 + `min_age_days` 过滤
- [x] **Step 3:** `rules_get_suggestions` 返回分页列表
- [x] **Step 4:** 前端 Cleanup 页接真实数据

---

### Task 4.2：QuarantineService

**Files:**

- Create: `src-tauri/src/services/quarantine.rs`
- Create: `src-tauri/src/commands/quarantine.rs`

- [x] **Step 1:** 移入隔离区：UUID 目录 + meta.json + DB insert
- [x] **Step 2:** 还原 + 冲突策略 overwrite/alternate
- [x] **Step 3:** 永久删除 + 确认文案校验
- [x] **Step 4:** 前端 Quarantine 页接真实 API

---

### Task 4.3：CleanupService + AuditService

**Files:**

- Create: `src-tauri/src/services/cleanup.rs`
- Create: `src-tauri/src/services/audit.rs`
- Create: `src-tauri/src/commands/cleanup.rs`, `audit.rs`

- [x] **Step 1:** `cleanup_execute` 编排：规则校验 → quarantine move → audit log
- [x] **Step 2:** `danger_confirm_token` 校验（含 danger 项时）
- [x] **Step 3:** `audit_list`, `audit_export`, `audit_clear`
- [x] **Step 4:** 前端 Cleanup / Audit 页收尾

---

## M5：双通道 AI

### Task 5.1：AiProvider 抽象

**Files:**

- Create: `src-tauri/src/services/ai/mod.rs`
- Create: `src-tauri/src/services/ai/cloud_deepseek.rs`
- Create: `src-tauri/src/services/ai/local_ollama.rs`
- Modify: `src-tauri/src/services/config.rs`

- [x] **Step 1:** 定义 trait `AiProvider { async fn chat(...) -> Result<String> }`
- [x] **Step 2:** `CloudDeepSeekProvider`：`POST https://api.deepseek.com/v1/chat/completions`，model=`deepseek-chat`，Key 从 keyring 读取
- [x] **Step 3:** `LocalOllamaProvider`：`POST {ollama_base}/v1/chat/completions`（Ollama OpenAI 兼容），model 默认 `deepseek-r1:1.5b`
- [x] **Step 4:** 配置项 `ai_mode`: `cloud` | `local`；`ollama_base_url` 默认 `http://127.0.0.1:11434`

**Ollama 预置命令（文档/脚本）:**

```bash
ollama pull deepseek-r1:1.5b
# 验证
curl http://127.0.0.1:11434/v1/chat/completions -d '{"model":"deepseek-r1:1.5b","messages":[{"role":"user","content":"hi"}]}'
```

---

### Task 5.2：AiService 与 Command

**Files:**

- Create: `src-tauri/src/services/ai/service.rs`
- Create: `src-tauri/src/commands/ai.rs`

- [x] **Step 1:** 共用 `desensitize_path()` + system prompt 组装
- [x] **Step 2:** `ai_chat` 按 `ai_mode` 路由 Provider
- [x] **Step 3:** `ai_test_connection`：cloud 测 Key；local 测 Ollama `GET /api/tags` 是否含目标模型
- [x] **Step 4:** 前端 Analysis 页去掉 mock；Settings AI Tab 接 `config_get/save`

**错误码:** `AiOllamaUnavailable`（Ollama 未启动）、`AiModelNotFound`（未 pull 模型）

---

## M6：ConfigService 与收尾

### Task 6.1：ConfigService 完整实现

**Files:**

- Modify: `src-tauri/src/services/config.rs`
- Modify: `src/pages/settings/SettingsPage.tsx`

- [x] **Step 1:** 全部设置字段读写（theme, quarantine, retention, thresholds, ai_mode, ollama_*）
- [x] **Step 2:** API Key → keyring；`has_api_key` 布尔
- [x] **Step 3:** `index_clear` 二次确认

---

### Task 6.2：跨页上下文与 polish

**Files:**

- Modify: `src/stores/selection-context.ts`
- Modify: 各页面 toast / loading / empty state

- [x] **Step 1:** 浏览/清理 → AI 上下文 Zustand 持久到 session
- [x] **Step 2:** 全局 ErrorBoundary + toast
- [x] **Step 3:** 空索引、扫描中、AI 不可用等状态全覆盖

---

### Task 6.3：v1 验收测试

**Files:**

- Create: `docs/current/modules/disk-helper/v1-acceptance-checklist.md`

- [x] **Step 1:** 按产品概要 AC-01～AC-13 逐条手工验收
- [x] **Step 2:** 真实 C 盘全量扫描一次（用户本机）
- [x] **Step 3:** 测试 cloud + local AI 各一问
- [x] **Step 4:** 软删除 → 隔离区 → 还原 闭环
- [x] **Step 5:** 打包：`npm run tauri build`，验证 exe 安装运行

```bash
npm run tauri build
# 产物: src-tauri/target/release/bundle/
```

---

## 规范与纪律


| 项      | 要求                                        |
| ------ | ----------------------------------------- |
| Commit | 每 Task 完成后 commit；原型 Checkpoint 单独 commit |
| 测试     | Rust 核心逻辑先写 test；前端关键组件 Vitest            |
| 安全     | 前端禁止直接 fs；API Key 不进 log/export           |
| 原型优先   | **M1 完成前不写 ScanService 真实 C 盘 walk**      |
| AI     | 规则引擎裁决清理；AI 只输出文本                         |


---

## Spec 覆盖自检


| 需求                         | 对应 Task       |
| -------------------------- | ------------- |
| C 盘总览                      | 1.3, 3.1, 3.3 |
| 空间浏览 Treemap/Top           | 1.4, 3.3      |
| 全量+增量扫描                    | 3.2, 3.4      |
| 规则清理+危险确认                  | 1.5, 4.1, 4.3 |
| 隔离区                        | 1.5, 4.2      |
| 云端 DeepSeek AI             | 5.1, 5.2      |
| **本地 Ollama DeepSeek 小模型** | 5.1, 5.2      |
| 操作日志                       | 1.6, 4.3      |
| 设置                         | 1.6, 6.1      |
| UI 原型体验                    | **M1 全部**     |
| 离线可用（无 AI）                 | 6.2           |


---

## 执行方式选择

Plan 已保存至 `docs/superpowers/plans/2026-06-22-disk-helper-v1.md`。

**建议执行顺序：**

1. **Subagent-Driven** — 每 Task 派发子 agent，Task 间 review（推荐）
2. **Inline Execution** — 本会话按 M0→M1 顺序执行，**M1 完成后暂停等你验收原型**

你想从 **M0 脚手架** 直接开始，还是先 **只做 M1 原型** 快速出可体验界面？