# Disk Helper

AI 磁盘管理助手 — Windows 本地桌面客户端，帮助理解 C 盘占用、安全清理与误删恢复。

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面壳 | Tauri 2 |
| 前端 | React 19 + TypeScript + Vite + Tailwind CSS v4 |
| 后端 | Rust + SQLite |
| AI | Ollama（本地）/ DeepSeek API（云端） |

## 项目结构

```
Disk_helper/
├── disk-helper/          # Tauri 桌面应用（前端 + Rust）
├── docs/
│   ├── current/          # 当前权威文档（PRD、详细设计等）
│   └── superpowers/      # 实现计划与工程历史
└── .cursor/              # Cursor 项目技能与规范
```

## 快速开始

环境要求见 [`docs/current/modules/disk-helper/dev-environment.md`](docs/current/modules/disk-helper/dev-environment.md)。

```powershell
cd disk-helper
.\scripts\setup-dev-env.ps1 -UseChinaMirror   # 首次安装依赖
npm run dev                                     # Web 原型 http://localhost:1420
npm run tauri dev                               # 桌面窗口
```

## 当前进度

- **M0** 脚手架与环境 ✅
- **M1** UI 原型（Mock 数据 + 毛玻璃界面）✅ — 暂停中
- **M2–M6** Rust IPC、真实扫描、规则引擎、AI、打包 — 待开发

实现计划：[`docs/superpowers/plans/2026-06-22-disk-helper-v1.md`](docs/superpowers/plans/2026-06-22-disk-helper-v1.md)

## 文档

| 文档 | 说明 |
|------|------|
| [产品概要说明书](docs/current/modules/disk-helper/产品概要说明书_v1.md) | 产品定位与功能范围 |
| [详细设计 v1](docs/current/modules/disk-helper/详细设计_v1.md) | 架构、表结构、Tauri Command |
| [PRD 系列](docs/current/modules/disk-helper/) | 各菜单需求文档 |

## 许可证

自用项目，暂未指定开源许可证。
