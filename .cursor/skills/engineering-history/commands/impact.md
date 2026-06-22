# impact — 变更影响分析

## 触发条件
- 用户输入 `/工程历史 impact 专题名` 或 `/工程历史 影响 专题名`

## 执行步骤

---

### 第一步：定位专题文档

**1.1 搜索索引**
- `Grep` 在 `docs/superpowers/Index.md` 中搜索专题名，`-i` 忽略大小写，输出模式 `content`
- 获取匹配行及其上下文（专题名、文档列表）

**1.2 专题定位**
若找到匹配：
- 提取该专题下的所有文档列表
- **命中则进入第二步**

**1.3 未找到专题**
若未找到匹配：
- 输出可用专题列表：
```
## 变更影响分析: {专题名}

未找到该专题。可用专题：
- Authentication
- Bookmark
- Browser
- Frontend Refactoring
- IronClaw
- SelfUpgrade
- Skill Management
- Startup
```
- 终止执行

---

### 第二步：汇总涉及范围

对定位到的每个文档 `Read` 全文，关注特定章节提取：

- **文件路径**：`src/`、`app-electron/`、`server-java/`、`docs/` 开头的路径
  - 正则：`(?:src|app-electron|server-java|docs)/[^\s\`"'")]+`
- **模块名**：从代码块、`@see`、`修改/新增文件` 章节提取 PascalCase 标识符
  - 正则：`[A-Z][a-z]+(?:[A-Z][a-zA-Z0-9]+)+`，排除英文单词
- **IPC channel**：从代码块提取 `` `xxx:yyy` `` 格式

---

### 第三步：输出影响报告

```
## 变更影响分析: {专题名}

### 时间跨度
- 起始/最新: YYYY-MM-DD | 文档数: N spec, M plan

### 变更脉络
1. {日期}: {标题} — {摘要}
2. ...

### 涉及模块（按出现频次排序）
| 模块/路径 | 出现次数 |
|-----------|----------|
| app-electron/ironclaw-manager.js | 6 |

### IPC Channel
- ironclaw:restart

### 关键文件
| 文件路径 | 文档 |
|----------|------|
| app-electron/ironclaw-manager.js | 架构设计, IPC 计划 |
```
