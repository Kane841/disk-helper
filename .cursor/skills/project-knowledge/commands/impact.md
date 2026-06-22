# impact — 变更影响分析

## 触发条件
- 用户输入 `/project-knowledge impact 模块名` 或 `/project-knowledge 影响 模块名`

## 执行步骤

---

### 第一步：定位模块文档

**1.1 搜索索引**
- `Grep` 在 `docs/current/Index.md` 中搜索模块名，`-i` 忽略大小写，输出模式 `content`
- 获取匹配行及其上下文（模块标题、文档列表）

**1.2 模块定位**
若找到匹配：
- 提取该模块下的所有文档列表
- **命中则进入第二步**

**1.3 未找到模块**
若未找到匹配：
- 输出可用模块列表：
```
## 变更影响分析: {模块名}

未找到该模块。可用模块：
- Authentication
- Browser
- Electron
- IronClaw Integration
- OnlineHub OpenAPI
- PageLayout
- Startup
- SelfUpgrade
- 规范
```
- 终止执行

---

### 第二步：汇总涉及范围

对定位到的每个文档 `Read` 全文，关注特定章节提取：

- **文件路径**：`src/`、`app-electron/`、`server-java/`、`docs/` 开头的路径
  - 正则：`(?:src|app-electron|server-java|docs)/[^\s\`"'")]+`
- **模块/类名**：从代码块、`@see`、`修改/新增文件` 章节提取 PascalCase 标识符
  - 正则：`[A-Z][a-z]+(?:[A-Z][a-zA-Z0-9]+)+`，排除英文单词
- **IPC Channel**：从代码块提取 `` `xxx:yyy` `` 格式或 `ipcMain.handle` 中的 channel 名称
- **数据库表**：`CREATE TABLE`、`ALTER TABLE`、`t_xxx` 格式
- **API 端点**：`@GetMapping`、`@PostMapping`、`@RequestMapping` 等注解中的路径

---

### 第三步：输出影响报告

```
## 变更影响分析: {模块名}

### 文档概览
- 文档数: N 个 (PRD: A, DESIGN: B, SPEC: C)
- 涉及: 前端 (DESIGN_FE), 后端 (DESIGN_BE), 规范 (SPEC)

### 文档列表
1. {文件描述} — {状态}。{摘要}
2. ...

### 涉及文件路径（按出现频次排序）
| 文件路径 | 出现次数 |
|----------|----------|
| server-java/src/... | 6 |
| app-electron/... | 4 |

### IPC Channel
- {channel_name} — 出现在 {文档名}

### 数据库表
| 表名 | 操作 | 文档 |
|------|------|------|
| t_xxx | CREATE, ALTER | {DESIGN_BE_*} |

### API 端点
| 方法 | 路径 | 文档 |
|------|------|------|
| GET | /api/xxx | {DESIGN_BE_*} |

### 关键类/模块
- {ClassName} — 出现在 {文档名}
```
