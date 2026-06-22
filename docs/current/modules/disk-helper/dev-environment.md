# Disk Helper 开发环境说明（Windows）

> 适用：M0 脚手架 + Tauri 2 桌面开发 + 后续 M1 前端原型

## 1. 必需组件

| 组件 | 版本建议 | 用途 | 验证命令 |
|------|----------|------|----------|
| Node.js | ≥ 20 LTS | 前端构建 | `node --version` |
| npm | ≥ 10 | 包管理 | `npm --version` |
| Rust stable | latest | Tauri 后端 | `rustc --version` |
| Cargo | 随 Rust | 构建 | `cargo --version` |
| MSVC Build Tools | VS 2022 | Windows 原生编译 | 见下文 |
| WebView2 | 系统自带或手动装 | Tauri 渲染 | Win10/11 通常已有 |

## 2. 一键安装（推荐）

在 **PowerShell** 中执行（**国内网络建议加 `-UseChinaMirror`**）：

```powershell
cd c:\WorkPlace\Java\Disk_helper\disk-helper
Set-ExecutionPolicy -Scope Process Bypass -Force
.\scripts\setup-dev-env.ps1 -UseChinaMirror
```

脚本会自动：

- 安装 Rust（rustup + rsproxy 镜像）
- 写入 `disk-helper/.cargo/config.toml`（Cargo crates 镜像）
- 执行 `npm install`

可选参数：

- `-SkipVsBuild` — 已安装 Visual Studio / Build Tools 时跳过
- `-SkipOllama` — 暂不安装 Ollama（M5 AI 阶段再装）
- `-UseChinaMirror` — Rust / npm 使用国内镜像

## 3. 手动安装

### 3.1 Rust

1. 打开 https://www.rust-lang.org/tools/install
2. 下载并运行 `rustup-init.exe`，选默认 stable
3. **重新打开终端**，执行 `rustc --version`

国内镜像（PowerShell 安装前设置）：

```powershell
$env:RUSTUP_DIST_SERVER = "https://rsproxy.cn"
$env:RUSTUP_UPDATE_ROOT = "https://rsproxy.cn/rustup"
```

### 3.2 Visual Studio C++ Build Tools

1. 下载 [Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. 安装时勾选 **「使用 C++ 的桌面开发」**（含 MSVC v143、Windows SDK）

或 winget：

```powershell
winget install -e Microsoft.VisualStudio.2022.BuildTools
```

### 3.3 项目依赖

```powershell
cd c:\WorkPlace\Java\Disk_helper\disk-helper
npm install --registry=https://registry.npmmirror.com
```

## 4. 可选：Ollama（M5 本地 AI）

```powershell
# 安装 Ollama: https://ollama.com/download
ollama pull deepseek-r1:1.5b
ollama list
```

## 5. 运行项目

### 5.1 仅前端（M1 原型阶段）

```powershell
cd c:\WorkPlace\Java\Disk_helper\disk-helper
npm run dev
```

浏览器访问：**http://localhost:1420**

### 5.2 Tauri 桌面模式

```powershell
cd c:\WorkPlace\Java\Disk_helper\disk-helper
npm run tauri dev
```

首次运行会编译 Rust 依赖，耗时较长，属正常现象。

### 5.3 打包

```powershell
npm run tauri build
```

产物：`disk-helper/src-tauri/target/release/bundle/`

## 6. 项目结构

```text
Disk_helper/                 # 仓库根（文档 + 技能）
└── disk-helper/             # Tauri 应用
    ├── src/                 # React 前端
    ├── src-tauri/           # Rust 后端
    ├── scripts/setup-dev-env.ps1
    └── package.json
```

## 7. 常见问题

| 现象 | 处理 |
|------|------|
| `rustc` 不是内部命令 | 安装 Rust 后**重开终端**，或把 `%USERPROFILE%\.cargo\bin` 加入 PATH |
| `link.exe` not found | 安装 VS C++ Build Tools |
| `npm install` 很慢 | 使用 `npm install --registry=https://registry.npmmirror.com` |
| rustup 下载失败 | 使用 `setup-dev-env.ps1 -UseChinaMirror` |
| `tauri dev` WebView2 报错 | 安装 [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) |
| 首次 `tauri dev` 极慢 | 正在编译 Rust crate，等待完成即可 |

## 8. 当前环境状态（M0）

| 项 | 状态 |
|----|------|
| Node.js v24 | ✅ |
| disk-helper 脚手架 | ✅ |
| npm 依赖 | ✅ |
| Rust stable 1.96 | ✅（rsproxy 安装） |
| Cargo 镜像 `.cargo/config.toml` | ✅ rsproxy sparse |
| `npm run build` | ✅ 通过 |
| `npm run tauri dev` | ✅ 首次编译约 3–4 分钟，窗口可启动 |
| VS Build Tools | ✅ 本机已可用（编译已通过） |
| Ollama | ⚠️ 可选，M5 前 `ollama pull deepseek-r1:1.5b` |
