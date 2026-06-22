---
name: project-knowledge
description: 通过 /project-knowledge 调用，维护与检索 docs/current/Index.md，提供 docs/current 权威文档的索引构建、关键词检索、变更影响分析能力。用于快速定位当前工程状态的文档知识库，理解项目各模块的设计、规范和需求。
---

# Project Knowledge — 当前项目状态知识库

`docs/current` 目录下的文档是当前工程的权威真源。**本技能对应的索引真源文件为 [`docs/current/Index.md`](../../../docs/current/Index.md)**（与 `docs/current/` 正文配套；`build` / `update` 即维护该文件）。

本 skill 提供四个命令：

| 命令 | 用途 | 指令文件 |
|------|------|----------|
| `build` | 全量重建 `docs/current/Index.md` | `commands/build.md` |
| `update` | 根据最近 git 变更增量更新索引 | `commands/update.md` |
| `search <keyword>` | 关键词检索权威文档 | `commands/search.md` |
| `impact <模块名>` | 分析某模块文档涉及的变更影响范围 | `commands/impact.md` |

## 使用方法

根据用户输入匹配到对应命令后，读取 `commands/` 目录下对应的指令文件并按其步骤执行。
