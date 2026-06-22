---
name: finishing-work
description: 使用当需要收尾当前所有工作、切换任务或结束会话前——按顺序触发文档归档、知识库更新和 commit 重整流程。
---

# 收尾当前所有工作

按顺序触发四个技能，完成工作的收尾流程：文档归档、知识库更新、Commit 重整。

## 何时使用

- 完成一个功能模块后需要收尾
- 切换到其他任务前
- 一天工作结束前
- 需要整理当前分支状态时

## 执行流程

```
收尾进度：
- [ ] Step 1: 触发 releasing-design-docs（设计文档归档）
- [ ] Step 2: 触发 engineering-history update（工程历史索引更新）
- [ ] Step 3: 触发 project-knowledge update（项目知识库索引更新）
- [ ] Step 4: 触发 reorganize-commits（Commit 重整并推送）
```

---

## Step 1: 设计文档归档

读取并执行 `releasing-design-docs` 技能：

1. 确认研发状态（代码已完成、测试已通过）
2. 识别 `docs/superpowers/specs/` 和 `docs/superpowers/plans/` 中需要归档的文档
3. 判断 Release 类型（新增或合入已有文档）
4. 执行文档归档到 `docs/current/` 对应目录
5. 在原文档添加归档状态标记

**禁止**：在代码未完成或测试未通过时进行 Release。

---

## Step 2: 工程历史索引更新

读取并执行 `engineering-history` 技能的 `update` 命令：

1. 读取 `engineering-history/commands/update.md`
2. 根据最近 git 变更增量更新 `docs/superpowers/Index.md`

---

## Step 3: 项目知识库索引更新

读取并执行 `project-knowledge` 技能的 `update` 命令：

1. 读取 `project-knowledge/commands/update.md`
2. 根据最近 git 变更增量更新 `docs/current/Index.md`

---

## Step 4: Commit 重整

读取并执行 `reorganize-commits` 技能：

1. **确认当前分支状态**
   ```bash
   git status
   git log --oneline -10
   ```

2. **分析 commit 历史**
   ```bash
   git log origin/<branch>..HEAD --oneline
   ```

3. **确认重整范围** — 向用户确认需要重整的 commit 范围

4. **执行 soft reset**
   ```bash
   git reset --soft <target-commit>
   ```

5. **按类别分批 commit**
   - 前端：`git add <前端工程目录>/` + `feat(fe):/fix(fe):/refactor(fe):`
   - 后端：`git add <后端工程目录>/` + `feat(be):/fix(be):/refactor(be):`
   - 文档：`git add docs/` + `docs:`
   - 其他：`git add <其他文件>` + `chore:`

6. **展示新 commit 历史** — 供用户确认

7. **用户确认后 push**

---

## 目录约定

> **通用化设计**：以下目录为占位符，实际路径需根据工程目录结构确定。

| 类别 | 路径占位符 | Commit 前缀 |
|------|------------|-------------|
| 前端 | `<前端工程目录>/` | `feat(<模块>-fe):` / `fix(<模块>-fe):` / `refactor(<模块>-fe):` |
| 后端 | `<后端工程目录>/` | `feat(<模块>-be):` / `fix(<模块>-be):` / `refactor(<模块>-be):` |
| 文档 | `docs/` | `docs(<模块>):` |
| 其他 | 根目录及其他 | `chore:` |

**常见工程目录参考**：

| 工程结构 | 前端目录 | 后端目录 |
|----------|----------|----------|
| Vue + Spring Boot | `frontend/` | `backend/` |
| Electron + Java | `app-electron/` | `server-java/` |
| React + Node | `src/` | `server/` |

---

## 注意事项

- 每个 Step 都需要用户交互确认后才能继续
- 如果某个 Step 不需要执行（如没有文档需要归档），可跳过
- Commit 重整前确保所有变更已提交或暂存
- 禁止对已 push 的 commit 进行重整（除非明确使用 force push）

---

## 关联技能

- [releasing-design-docs](./releasing-design-docs/SKILL.md)
- [engineering-history](./engineering-history/SKILL.md)
- [project-knowledge](./project-knowledge/SKILL.md)
- [reorganize-commits](./reorganize-commits/SKILL.md)
