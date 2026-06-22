---
name: reorganize-commits
description: 在 push 前重新整合 commit，将前端和后端代码分开提交。使用当用户要 push 代码、需要整理 commit 历史、或将混杂的 commit 按前后端分离时。
---

# Commit 重整与 Push

在 push 代码前，将当前分支的 commit 按模块分类重新整合。

## 适用场景

- 执行完 Superpowers plan 后，commit 历史杂乱
- 一个 commit 中混合了多个模块的代码
- 需要 clean 的 commit 历史后再 push

## 前缀格式

### 格式规则

| 类型 | 格式 | 示例 |
|------|------|------|
| 前端代码 | `feat/fix/refactor(<模块>-fe):` | `feat(device-fe):` `fix(noti-fe):` |
| 后端代码 | `feat/fix/refactor(<模块>-be):` | `feat(device-be):` `fix(noti-be):` |
| 文档 | `docs(<模块>):` | `docs(device):` `docs(noti):` |
| 其他 | `chore:` | `chore:` |

### 常用模块标识

| 模块标识 | 含义 | 对应代码目录 |
|------|------|-------------|
| `device` | 设备管理模块 | 前端工程中设备相关目录 |
| `noti` | 通知模块 | 前端工程中通知相关目录 |
| `browser` | 浏览器模块 | 前端工程中浏览器相关目录 |
| `auth` | 认证模块 | 认证相关代码目录 |
| `upgrade` | 升级模块 | 升级相关代码目录 |
| `ironclaw` | IronClaw 模块 | IronClaw 相关代码目录 |

> **约定**：
> - 代码类 commit 必须带模块标识，避免使用 `feat(fe):` 或 `fix(be):` 无模块前缀形式。
> - 具体代码路径需根据实际工程目录结构确定，表格中仅为参考位置。

---

## 执行流程

```
重整进度：
- [ ] Step 1: 确认当前分支状态
- [ ] Step 2: 分析 commit 历史
- [ ] Step 3: 确认重整范围
- [ ] Step 4: 执行 soft reset
- [ ] Step 5: 按模块分批 commit
- [ ] Step 6: 展示新 commit 历史
- [ ] Step 7: 用户确认后 push
```

### Step 1: 确认当前分支状态

```bash
git status
git log --oneline -10
```

确认：
- 当前分支名
- 未提交的变更（如有，先处理）
- 远程分支状态

### Step 2: 分析 commit 历史

查看需要重整的 commit 范围：

```bash
# 查看与远程分支的差异
git log origin/<branch>..HEAD --oneline

# 或查看最近 N 个 commit
git log --oneline -N
```

分析每个 commit 涉及的文件和模块归属：

```bash
# 查看 commit 涉及的文件
git show --stat <commit-hash>
```

### Step 3: 确认重整范围

向用户确认：
1. 需要重整的 commit 范围（从哪个 commit 开始）
2. 是否有需要保留原样的 commit（如已审核通过的）

**交互式确认：**
```
发现以下 commit 需要重整：
- abc123: feat: xxx (涉及 device-fe/, noti-fe/)
- def456: fix: yyy (涉及 device-be/)

建议重整为：
- feat(device-fe): 设备管理前端实现
- feat(noti-fe): 通知模块前端实现
- fix(device-be): 设备管理后端修复
- docs(device): 设备管理设计文档
- docs(noti): 通知模块设计文档

是否继续？[Y/n]
```

### Step 4: 执行 soft reset

保留所有变更，撤回 commit：

```bash
# Reset 到目标 commit（保留文件变更）
git reset --soft <target-commit>
```

此时所有变更都在 staging area，等待重新 commit。

### Step 5: 按模块分批 commit

按顺序执行：

#### 5.1 重置 staging area

```bash
git reset HEAD
```

所有文件变更为 unstaged 状态。

#### 5.2 按模块分批 commit

按模块顺序依次 commit（前端模块 → 后端模块 → 文档）：

```bash
# 示例：按模块分批（路径需根据实际工程目录结构调整）
git add <前端工程目录>/src/services/...   # device-fe
git commit -m "fix(<模块>-fe): <描述>

变更内容：
- <文件>: <具体变更>"

git add <前端工程目录>/<通知相关目录>/   # noti-fe
git commit -m "feat(<模块>-fe): <描述>

变更内容：
- <文件>: <具体变更>"

git add <后端工程目录>/...                 # device-be / noti-be
git commit -m "fix(<模块>-be): <描述>

变更内容：
- <文件>: <具体变更>"

git add docs/...                           # docs
git commit -m "docs(<模块>): <描述>

变更文档：
- <文档路径>: <变更说明>"
```

### Step 6: 展示新 commit 历史

```bash
git log --oneline -N
```

向用户展示重整后的 commit 历史，确认是否符合预期。

### Step 7: 用户确认后 push

```
新 commit 历史：
- abc001: fix(device-fe): 设备管理前端实现
- abc002: feat(noti-fe): 通知模块前端实现
- abc003: fix(device-be): 设备管理后端修复
- abc004: docs(device): 设备管理设计文档
- abc005: docs(noti): 通知模块设计文档

确认 push？[Y/n]
```

用户确认后执行：

```bash
git push origin <branch>
```

---

## Commit Message 模板

### 前端代码

```
<type>(<模块>-fe): <简短描述>

变更内容：
- <文件/模块>: <具体变更>
- <文件/模块>: <具体变更>

影响范围：<页面/组件/功能>
```

### 后端代码

```
<type>(<模块>-be): <简短描述>

变更内容：
- <类/文件>: <具体变更>
- <类/文件>: <具体变更>

API 变更：<如有，列出接口路径>
```

### 文档

```
docs(<模块>): <简短描述>

变更文档：
- <文档路径>: <变更说明>
- <文档路径>: <变更说明>
```

---

## 类型映射

| 原类型 | 前端 | 后端 | 文档 |
|--------|------|------|------|
| feat | feat(<模块>-fe) | feat(<模块>-be) | docs(<模块>) |
| fix | fix(<模块>-fe) | fix(<模块>-be) | docs(<模块>) |
| refactor | refactor(<模块>-fe) | refactor(<模块>-be) | docs(<模块>) |
| test | test(<模块>-fe) | test(<模块>-be) | - |
| chore | chore | chore | chore |

---

## 注意事项

### 禁止事项

- ❌ 对已 push 的 commit 进行重整（除非明确使用 force push）
- ❌ 在 main/master 分支执行重整（需用户明确授权）
- ❌ 丢弃任何代码变更（soft reset 保留所有变更）
- ❌ 跳过用户确认直接 push
- ❌ 代码 commit 不带模块标识（如 `feat(fe):` 而非 `feat(device-fe):`）

### 安全检查

执行前确认：
1. 当前分支不是 main/master（或用户明确授权）
2. 所有变更都已保存（无未暂存的修改）
3. 远程分支状态已知（是否需要 force push）

### Force Push 警告

如果已 push 过需要 force push：

```
警告：此操作需要 force push，会覆盖远程历史。
确认继续？[Y/n]
```

用户确认后使用：
```bash
git push --force-with-lease origin <branch>
```

---

## 示例

**原始 commit 历史：**
```
a1b2c3: feat: IronClaw 配置功能
d4e5f6: fix: 修复通知 API 响应格式
g7h8i9: docs: 更新设计文档
```

**分析：**
- a1b2c3: 混合了 ironclaw-fe 和 ironclaw-be
- d4e5f6: 仅 noti-be
- g7h8i9: 仅 docs（ironclaw 模块）

**重整后：**
```
001aaa: feat(ironclaw-fe): IronClaw 配置前端实现

变更内容：
- IronClawConfigSection.vue: 新增配置组件
- ironclaw-config.js: 新增 API 模块

002bbb: feat(ironclaw-be): IronClaw 配置后端实现

变更内容：
- IronClawConfigController.java: 新增配置接口
- IronClawConfigService.java: 配置服务实现

API 变更：GET/POST /api/v1/ironclaw/config

003ccc: fix(noti-be): 修复通知 API 响应格式

变更内容：
- NotificationController.java: 修复响应结构

004ddd: docs(ironclaw): 更新 IronClaw 配置设计文档

变更文档：
- DESIGN_IronClaw_Settings.md: 补充 API 说明
```
