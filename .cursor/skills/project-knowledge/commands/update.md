# update — 增量更新索引

## 触发条件
- 用户输入包含 `update`、`更新` 等关键词

## 执行步骤

---

### 第一步：检查索引状态

**1.1 检查 Index.md 是否存在**
- 检查 `docs/current/Index.md` 是否存在
- 不存在则输出：`请先执行 /project-knowledge build 构建索引` 并终止

**1.2 获取最近变更**
- `Bash` 执行 `git log --diff-filter=ACMRT --name-only --pretty=format: -- "docs/current/*" --since="3 days ago"` 获取新增/修改文件（不限制数量）
- 另执行 `git log --diff-filter=D --name-only --pretty=format: -- "docs/current/*" --since="3 days ago"` 获取已删除文件

---

### 第二步：定位需要更新的模块

**2.1 搜索变更模块**
- 对变更文件列表，`Grep` 在 `docs/current/Index.md` 中搜索匹配
- 确定每个变更文件属于哪个模块

**2.2 提取更新内容**
对变更文档重新执行 build Step 2 提取元数据：
- **标题**：文件第一个 H1 的内容（去除 `# `）
- **类型**：从文件名前缀提取
- **模块**：从目录路径提取
- **状态**：从 `> 文档状态：xxx` 行提取
- **摘要**：截取 100 字

---

### 第三步：更新索引

**3.1 读取现有索引**
- `Read` `docs/current/Index.md` 完整内容

**3.2 执行更新**
- **新增**：添加对应模块条目，无匹配模块则新增模块
- **修改**：更新摘要和关联链接
- **删除**：移除对应条目
- 更新头部时间和统计：`> 自动生于 {YYYY-MM-DD HH:mm}`，`> 共 N 个文档，M 个模块`

**3.3 写回索引**
- 将更新后的内容写回 `docs/current/Index.md`

---

### 第四步：输出报告

```
索引增量更新完成：N 个文档（新增 A，修改 B，删除 C）
```

若 0 个变更则输出：`最近无文档变更，索引无需更新。`
