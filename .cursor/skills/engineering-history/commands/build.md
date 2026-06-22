# build — 全量重建索引

## 触发条件
- 用户输入包含 `build`、`构建`、`重建索引` 等关键词

## 执行步骤

**1. 扫描文档**
- `Glob` 扫描 `docs/superpowers/specs/*.md` 和 `docs/superpowers/plans/*.md`
- 排除 `.gitkeep`

**2. 提取元数据**
对每个文档 `Read` 前 50 行：
- **标题**：文件第一个 H1 的内容（去除 `# `），若无 H1 则用文件名去除日期前缀
- **日期**：文件名中的 `YYYY-MM-DD` 部分
- **类型**：路径含 `specs/` 为 Spec，含 `plans/` 为 Plan
- **摘要**：跳过 frontmatter/YAML 头，取第一个非标题普通段落，截取 150 字

**3. 专题归类**（按顺序匹配，命中即停）
含 IronClaw → IronClaw；Browser → Browser；bookmark → Bookmark；Auth → Authentication；Upgrade → SelfUpgrade；Skill → Skill Management；Startup → Startup；Layout/Refactoring/重构 → Frontend Refactoring；其余 → Other

**4. 关联**：对比文件名核心部分（去除日期和类型后缀），建立 Spec-Plan 双向链接

**5. 生成索引**，写入 `docs/superpowers/Index.md`：

```markdown
# Superpowers 工程历史索引

> 自动生于 {YYYY-MM-DD HH:mm}
> 共 N 个 spec，M 个 plan，K 个专题

---

## 专题: Authentication

- [2026-03-30 认证 Token 存储简化设计](specs/2026-03-30-Auth-Token-Simplification.md) — token 存储从 localStorage 转移到后端数据库。
  - <details><summary>详情</summary>
    - 类型: Spec
    - 关联文档: [实现计划](plans/2026-03-30-Auth-Token-Simplification.md)
    </details>

## 专题: IronClaw

- [2026-03-30 IronClaw 配置页面设计](specs/2026-03-30-IronClaw-Settings-Page.md) — LLM 推理提供商、沙箱开关、重启进程。
  - <details><summary>详情</summary>
    - 类型: Spec
    - 关联文档: [后端计划](plans/2026-03-31-IronClaw-Settings-Backend.md)、[前端计划](plans/2026-03-31-IronClaw-Settings-Frontend-Infrastructure.md)
    </details>
```

**条目格式规则**：
- 基础：`- [{日期} {标题}](路径) — {摘要}`
- 有关联：折叠详情块展示类型和关联，多个关联用顿号分隔
- 无关联：只写基础条目，不加折叠块
- 专题内日期升序，专题间按字母排序

**6. 输出报告**：`索引构建完成：N 个 spec, M 个 plan, K 个专题`。若 0 个文件输出 `当前无 spec 或 plan 文档，索引为空。`
