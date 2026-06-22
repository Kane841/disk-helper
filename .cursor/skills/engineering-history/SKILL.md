---
name: engineering-history
description: 通过 /engineering-history 调用，维护与检索 docs/superpowers/Index.md，提供工程历史（specs/plans）索引构建、关键词检索、变更影响分析能力。用于追溯项目历史变更、了解某个模块的演进脉络。
---

# Engineering History Index

`docs/superpowers/` 目录下的 spec 和 plan 文档记录了项目演进历史。**本技能对应的索引真源文件为 [`docs/superpowers/Index.md`](../../../docs/superpowers/Index.md)**（与 `specs/`、`plans/` 配套；`build` / `update` 即维护该文件）。

本 skill 提供四个命令：

| 命令 | 用途 | 指令文件 |
|------|------|----------|
| `build` | 全量重建 `docs/superpowers/Index.md` | `commands/build.md` |
| `update` | 根据最近 git 变更增量更新索引 | `commands/update.md` |
| `search <keyword>` | 关键词检索历史文档 | `commands/search.md` |
| `impact <topic>` | 分析某专题的变更影响范围 | `commands/impact.md` |

## 使用方法

根据用户输入匹配到对应命令后，读取 `commands/` 目录下对应的指令文件并按其步骤执行。
