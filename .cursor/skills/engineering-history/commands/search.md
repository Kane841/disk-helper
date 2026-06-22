# search — 关键词检索

## 触发条件
- 用户输入 `/工程历史 search 关键词` 或 `/工程历史 检索 关键词`

## 执行步骤

---

### 第一步：直接搜索索引

**1.1 搜索 Index.md**
- `Grep` 在 `docs/superpowers/Index.md` 中搜索关键词，`-i` 忽略大小写，输出模式 `content`
- 获取匹配行及其上下文（专题名、文档标题、日期、摘要）

**1.2 索引命中处理**
若找到匹配：
- 提取匹配文档的：文件路径、日期、类型、关联文档
- 评分规则：
  - 标题匹配 ★★★★
  - 摘要匹配 ★★★
- **直接进入第四步输出**

---

### 第二步：转换关键字再搜索索引

**2.1 关键字转换**
根据关键词类型进行同义扩展：
- 英文 → 中文（如 `IronClaw` → `铁爪`）
- 中文 → 英文（如 `认证` → `Auth/Authentication`，`配置` → `Settings`）
- 功能描述 → 专题名（如 `聊天` → `IronClaw`，`技能` → `Skill Management`）

**2.2 再次搜索 Index.md**
- 用转换后的关键字重新执行 `Grep` 搜索 `docs/superpowers/Index.md`
- 若找到匹配，按 1.2 规则评分处理
- **命中则进入第四步输出**

---

### 第三步：阅读索引理解匹配

**3.1 读取 Index.md 全文**
- `Read` 完整读取 `docs/superpowers/Index.md`

**3.2 语义匹配**
理解用户搜索意图，在索引中寻找语义相关的专题：
- 专题名相关：Authentication、IronClaw、Browser、Bookmark、SelfUpgrade 等
- 功能相关：如"Token" 匹配 Auth 专题、"LLM/推理" 匹配 IronClaw 专题

**3.3 评分输出**
- 专题名精确匹配 ★★★★
- 功能相关匹配 ★★★
- **命中则进入第四步输出**

---

### 第四步：全文搜索兜底（最终兜底）

**4.1 全文搜索**
- `Grep` 在 `docs/superpowers/specs/` 和 `docs/superpowers/plans/` 中搜索关键词，`-i` 忽略大小写
- 输出模式 `files_with_matches`

**4.2 收集详情**
- `Read` 匹配文件前 30 行获取标题、日期
- `Grep -C 2` 获取关键词上下文

**4.3 评分**
- 标题匹配 ★★★★
- 内容匹配 ★★★

---

### 第五步：输出结果

若无匹配：
```
## 检索结果: "{keyword}"

未找到匹配文档。

建议：
- 尝试其他关键词
- 运行 /工程历史 build 查看所有专题
```

若有匹配（来源：索引/语义匹配/全文）：
```
## 检索结果: "{keyword}"

找到 N 个匹配文档：

### 1. ★★★★ 认证 Token 存储简化设计
|- 文件: [规格](specs/2026-03-30-Auth-Token-Simplification.md) | [计划](plans/2026-03-30-Auth-Token-Simplification.md)
|- 日期: 2026-03-30
|- 类型: Spec
|- 摘要: token 存储从 localStorage 转移到后端数据库。
```

按 ★ 等级降序，同等级按日期降序排列。
