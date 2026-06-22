# update — 增量更新索引

## 触发条件
- 用户输入包含 `update`、`更新` 等关键词

## 执行步骤

---

### 第一步：检查索引状态

**1.1 检查 Index.md 是否存在**
- 检查 `docs/superpowers/Index.md` 是否存在
- 不存在则输出：`请先执行 /工程历史 build 构建索引` 并终止

**1.2 获取最近变更**
- `Bash` 执行 `git log --diff-filter=ACMRT --name-only --pretty=format: -- "docs/superpowers/specs/*" "docs/superpowers/plans/*" --since="3 days ago"` 获取新增/修改文件（不限制数量）
- 另执行 `git log --diff-filter=D --name-only --pretty=format: -- "docs/superpowers/specs/*" "docs/superpowers/plans/*" --since="3 days ago"` 获取已删除文件

---

### 第二步：定位需要更新的专题

**2.1 搜索变更专题**
- 对变更文件列表，`Grep` 在 `docs/superpowers/Index.md` 中搜索匹配
- 确定每个变更文件属于哪个专题

**2.2 提取更新内容**
对变更文档重新执行 build Step 2 提取元数据：
- **标题**：文件第一个 H1 的内容（去除 `# `），若无 H1 则用文件名去除日期前缀
- **日期**：文件名中的 `YYYY-MM-DD` 部分
- **类型**：路径含 `specs/` 为 Spec，含 `plans/` 为 Plan
- **摘要**：截取 150 字

---

### 第三步：更新索引

**3.1 读取现有索引**
- `Read` `docs/superpowers/Index.md` 完整内容

**3.2 执行更新**
- **新增**：添加对应专题条目，无匹配专题则新增专题
- **修改**：更新摘要和关联链接
- **删除**：移除对应条目
- 更新头部时间和统计：`> 自动生于 {YYYY-MM-DD HH:mm}`，`> 共 N 个 spec, M 个 plan, K 个专题`

**3.3 写回索引**
- 将更新后的内容写回 `docs/superpowers/Index.md`

---

### 第四步：输出报告

```
索引增量更新完成：N 个（新增 A，修改 B，删除 C）
```

若 0 个变更则输出：`最近无 spec 或 plan 文档变更，索引无需更新。`
