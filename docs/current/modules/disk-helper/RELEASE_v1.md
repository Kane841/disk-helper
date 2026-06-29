# Disk Helper v1.0 发布说明

> 文档状态：已发布 · 发布日期：2026-06-23 · Git tag：`v1.0.0`

## 交付范围

Windows 本地桌面版（Tauri 2 + React + Rust），单机 C 盘空间管理闭环：

- 磁盘总览、全量/增量扫描、空间浏览（目录树 / Treemap / Top 榜单）
- 规则驱动安全清理、隔离区软删除与还原、操作日志
- 双通道 AI（本地 Ollama + 云端 DeepSeek），规则引擎裁决清理行为
- 设置持久化、主题、清空索引

## 安装与运行

```powershell
# 开发
cd disk-helper
npm run tauri dev

# 发布包
npm run tauri build
# NSIS: src-tauri/target/release/bundle/nsis/Disk Helper_0.1.0_x64-setup.exe
# MSI:  src-tauri/target/release/bundle/msi/Disk Helper_0.1.0_x64_en-US.msi
```

应用版本号 `0.1.0`（`package.json` / `tauri.conf.json`），产品里程碑为 **v1.0**。

## 验收

- 清单：[v1-acceptance-checklist.md](./v1-acceptance-checklist.md)（AC-01～AC-13 已通过）
- 实现计划：[2026-06-22-disk-helper-v1.md](../../superpowers/plans/2026-06-22-disk-helper-v1.md)（M0–M6 已完成）

## 已知限制（v1.0）

- 仅支持 **C 盘**扫描与总览（`SCAN_ROOT` 可改，UI 未暴露）
- 增量扫描为 mtime/size 浅层 diff（depth≤2）
- AI 为固定结构回复，无多轮会话与读文件能力
- 安装包构建依赖 WiX / NSIS 工具链（首次 build 需下载或离线预置）

## 下一版本方向（v1.1+ 规划入口）

在 v1.0 基线上扩展，具体需求待新 PRD/计划：

- AI 体验增强（多轮、上下文、prompt 放宽）
- 扫描/性能与 UX 持续优化
- 可选：多盘符、规则可配置化

权威设计仍以 [详细设计_v1.md](./详细设计_v1.md)、[产品概要说明书_v1.md](./产品概要说明书_v1.md) 为基线；v1.1 变更请新增 plan/spec，合入或衍生 DESIGN 文档。
