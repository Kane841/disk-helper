# build — 全量重建索引

## 触发条件
- 用户输入包含 `build`、`构建`、`重建索引` 等关键词

## 执行步骤

**1. 扫描文档**
- `Glob` 扫描 `docs/current/**/*.md`
  - `docs/current/common/**/*.md` — 通用模块
  - `docs/current/modules/**/*.md` — 业务模块
  - `docs/current/SPEC_*.md` — 顶层规范
- 排除 `Index.md` 本身和 `.gitkeep`

**2. 提取元数据**
对每个文档 `Read` 前 30 行：
- **标题**：文件第一个 H1 的内容（去除 `# `）
- **类型**：从文件名前缀提取 `PRD` / `DESIGN` / `DESIGN_BE` / `DESIGN_FE` / `SPEC`
- **模块**：从目录路径提取。`common/Xxx/` → `Xxx`；`modules/Xxx/` → `Xxx`；顶层 `SPEC_*.md` → `规范`
- **状态**：从 `> 文档状态：xxx` 行提取（如"已完成"、"已归档"）；若无则标注 `未知`
- **摘要**：跳过 meta 区块（以 `>` 开头的行），取第一个普通段落（非标题、非表格、非代码块），截取 100 字

**3. 归类逻辑**

文档按模块名分组，模块按英文字母顺序排序（`规范` 排最后）。模块内文档按文件名自然排序。

**4. 文档关联**
- PRD ↔ DESIGN_BE / DESIGN_FE 关联：同一模块下同主题的文档互为关联
- DESIGN_FE ↔ DESIGN_BE 关联：同主题的前后端文档互为关联（文件名中包含相同主题词，排除 `_FE_`/`_BE_` 后比较）

**5. 生成索引**，写入 `docs/current/Index.md`：

输出格式参考 `docs/superpowers/Index.md`，每条文档使用 `<details>` 折叠块展示类型和关联文档：

```markdown
# Docs/Current 当前知识库索引

> 自动生于 {YYYY-MM-DD HH:mm}
> 共 N 个文档，M 个模块

---

## 模块: Authentication

- [PRD DAS-Pilot 登录认证功能需求](common/Authentication/PRD_Authentication.md) — 状态: 已批准。OAuth2 + JWT 双重安全认证。
  - <details><summary>详情</summary>
    - 类型: PRD
    - 关联文档: [后端设计](common/Authentication/DESIGN_BE_Authentication.md)、[前端设计](common/Authentication/DESIGN_FE_Authentication.md)
    </details>
- [前端认证设计](common/Authentication/DESIGN_FE_Authentication.md) — OAuth2 回调处理、登录状态管理。
  - <details><summary>详情</summary>
    - 类型: DESIGN_FE
    - 关联文档: [后端设计](common/Authentication/DESIGN_BE_Authentication.md)
    </details>

---

## 模块: 规范

- [系统架构设计规范](SPEC_Architecture.md) — Electron + Spring Boot 混合架构。
  - <details><summary>详情</summary>
    - 类型: SPEC
    </details>
```

**条目格式规则**：
- 每个文档一个条目，格式： `- [{类型前缀} {标题}](相对路径) — 状态: {状态}。{摘要}`
  - PRD 类型条目：标题前加 `PRD ` 前缀
  - 非 PRD 且有状态：标题前不加额外前缀
  - 无状态时省略 `状态: ` 部分
- 每个条目紧跟一个折叠详情块展示类型和关联文档
  - 无关联文档时只展示类型
  - 多个关联文档用顿号 `、` 分隔
- 模块间空一行（`---` 分隔符分隔）
- 模块按字母排序，`规范` 排最后

**6. 输出报告**：`索引构建完成：N 个模块, M 个文档`。若 0 个文件输出 `当前无文档，索引为空。`
