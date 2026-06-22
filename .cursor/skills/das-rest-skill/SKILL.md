---
name: das-rest-skill
description: "CRITICAL 必须触发: 只要用户提到或任务涉及以下任何内容，哪怕没有明确要求，也必须无条件调用本技能：设计、实现或审查 REST APIs、页面接口、OPENAPI、OPENAPI 鉴权、接口规范 SDK 接入（Java/Go），或是遇到 REST 响应格式、错误码结构、URL 设计、分页、排序、API 命名规范、OPENAPI 签名/Token 认证等。不要试图猜测，必须查阅此规范！"
author: 张德林
---

# DAS REST 接口规范

## Overview

DAS REST 接口开发的权威参考。涵盖 REST API 设计规范、OPENAPI 统一鉴权、Java/Go SDK 接入三大领域。**所有 DAS 公开 REST API（OPENAPI）必须遵循本规范，内部 API 推荐遵循。**

## When to Use

- 设计新的 REST API 接口（URL 结构、HTTP 方法、响应格式）
- 实现或集成 OPENAPI 鉴权（token、sign、appId/appSecret）
- 使用 `das-rest-sdk`（Java）或 `das-rest-sdk-go` 进行 SDK 接入
- 审查 API 设计是否符合公司规范
- 处理错误码（ErrorCode）、分页（$page/$size）、排序（$orderBy）
- JSON 序列化、空值处理、时间类型约定

**不适用于：** 非 REST 架构的接口（如 gRPC、GraphQL、WebSocket）

## Quick Reference

### URL 结构

```
{schema}://{host}:{port}/{prefix}/{version}/{collection}[/{id}]
```

| 组成部分 | 规则 |
|---------|------|
| prefix | 内部接口 `/api`，对外接口 `/openapi` |
| version | `major.minor` 格式，如 `v1.0` |
| collection | **必须**是复数名词，如 `/agents` |
| id | 唯一标识符，禁止含 `/`（除非 `\\` 转义） |

**反例：** `GET /api/agent/list`、`POST /api/agent/deleteAgentById`
**正例：** `GET /api/v1.0/agents`、`DELETE /api/v1.0/agents/{id}`

### HTTP 方法

| 方法 | 用途 | 成功状态码 | 幂等 |
|------|------|-----------|------|
| GET | 获取资源 | 200 | ✔ |
| POST | 创建资源 | 201 | ✘ |
| PUT | 替换资源（完整） | 200 | ✔ |
| PATCH | 更新资源（部分） | 200 | ✘ |
| DELETE | 删除资源 | 204 | ✔ |

### 成功响应

**必须**包含 `data` 键。单资源为对象，多资源为数组。

```json
// 单个资源
{ "data": { "agentType": "soc", "softVersion": "3.0.1" } }

// 多个资源
{ "data": [{ "agentType": "soc" }, { "agentType": "ainta" }] }

// 空集合
{ "data": [] }
```

### 错误响应

**必须**包含 `error` 键，`error` 对象**必须**含 `code` 和 `message`。

```json
{
  "error": {
    "code": "BadArgument",
    "message": "参数错误",
    "target": "password",
    "innerError": { "code": "PasswordDoesNotMeetPolicy" }
  }
}
```

### 标准错误码

| ErrorCode | HTTP 状态码 | 描述 |
|-----------|-----------|------|
| BadArgument | 400 | 参数无效 |
| BadUserArgument | 400 | 用户参数无效 |
| ErrorUnsupportedOrderBy | 400 | 不支持排序/不支持指定字段排序 |
| ErrorUnsupportedPaging | 400 | 不支持分页 |
| InvalidOperation | 400 | 请求在当前上下文中无效 |
| Unauthorized | 401 | 未授权 |
| InvalidApiKey | 401 | 无效 apiKey |
| NotFound | 404 | 资源不存在 |
| RequestTimeout | 408 | 操作超时 |
| Conflict | 409 | 资源冲突（POST 已存在/PATCH 不存在） |
| Overlimit | 409 | 超出系统限制 |
| InternalError | 500 | 服务端内部错误 |
| ServiceUnavailable | 503 | 服务不可用 |
| GatewayTimeout | 504 | 网关超时 |

### OPENAPI 鉴权错误码

| ErrorCode | HTTP 状态码 | 描述 |
|-----------|-----------|------|
| BadArgument | 400 | 参数无效 |
| BadTimeArgument | 400 | 时间参数无效 |
| BadSignArgument | 400 | 签名参数无效 |
| InvalidToken | 401 | Token 无效 |
| NotFound | 404 | appId 不存在 |

### 分页与排序

```
GET /api/v1.0/agents?$page=5&$size=10&$orderBy=name desc,createTime
```

响应**必须**回显参数并包含 `total`：

```json
{
  "$page": 5, "$size": 10, "$orderBy": "name desc,createTime",
  "data": [...], "total": 100
}
```

### JSON 标准化

| 类型 | 空值约束 |
|------|---------|
| String | `""` |
| Number | `0` |
| Boolean | 不允许空值 |
| Object | `{}` |
| Array | `[]` |

- 时间**必须**使用 Unix 毫秒时间戳（如 `1611469304000`）
- 持续时长使用 ISO 8601 格式（如 `P3Y6M4DT12H30M5S`）
- 64 位大整数**必须**作为字符串返回

### 命名规范

| 规则 | 示例 |
|------|------|
| 属性名使用 lowerCamelCase | `agentName`、`createTime` |
| 标识字段使用字符串类型 | `id`、`subscriptionId` |
| 日期时间后缀 DateTime | `createDateTime` |
| 仅日期后缀 Date | `birthDate` |
| 仅时间后缀 Time | `startTime` |
| 名称后缀 Name | `agentName` |
| 计数后缀 Count | `bookCount` |
| 集合使用复数 | `schemas`（非 `schemata`） |
| **禁止**使用命名 | `Context`、`Scope`、`Resource` |

### 通用子资源

| 子资源 | 用途 | 示例 |
|--------|------|------|
| files | 导入/导出 | `GET /agents/{id}/files` |
| synchronizations | 数据同步 | `POST /assets/synchronizations` |
| connectivities | 连通性检测 | `POST /assets/{id}/connectivities` |
| restorations | 重置 | `POST /menus/restorations` |
| options | 选项列表 | `GET /menus/options` |

## OPENAPI Authentication

鉴权流程三步走：

1. **获取 token**：客户端用 `appId` + `sign` + `timestamp` 调用 `GET /openapi/v1.0/app-tokens`
   - `sign = SHA256(timestamp + appId + appSecret)`
2. **调用 OPENAPI**：携带 Header（`timestamp`、`token`、`nonce`、`sign`）
   - `sign = SHA256(paramPairStr + "&" + timestamp + token + nonce)`
   - `paramPairStr`：非空 Query 参数按 ASCII 字典序排列拼接（`key1=value1&key2=value2`）

**关键规则：**
- **appSecret 不参与实际请求**，仅用于获取 token 时生成签名
- **只有 Query 参数参与签名**，Body 参数不参与
- 空参数值不参与签名；参数值需 trim
- 数组参数取最后一个元素；参数名区分大小写
- Hash 算法为 **SHA-256**，输出 16 进制字符串
- nonce 为 6 位随机字符串，防重放

> 详细鉴权规范参见 `references/das-openapi-auth.md`

## SDK Integration

### Java SDK (`das-rest-sdk`)

```xml
<!-- Maven 依赖（JDK8/11/17 可选） -->
<dependency>
  <groupId>com.dbapp.rest</groupId>
  <artifactId>das-rest-sdk</artifactId>
  <version>1.0.1-jdk17</version>
</dependency>
```

- 启动注解 `@EnableDasRest`
- 成功响应：`SuccessCode.Success.data(obj).build()`
- 错误响应：`ErrorCode.BadArgument.build()` 或 `ErrorResponse.error(...).message(...).build()`
- 分页：方法参数加 `Page page`、`Sort sort`
- 鉴权：实现 `IOpenApiToken` + `IOpenApiCheck`

> 详细指南参见 `references/das-rest-sdk-java-guide.md`

### Go SDK (`das-rest-sdk-go`)

```shell
go get gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go@v1.0.1
```

- Handler 包装：`handler.Handler(myHandler)` / `handler.PageHandler(myHandler)`
- 成功响应：直接返回对象或 `rest.Success(data).Build()`
- 错误响应：`rest.BadArgument.Message("msg").Build()`
- 分页排序：方法签名加 `page *rest.Page, sort *rest.Sort`
- 鉴权服务端：实现 `auth.AppManager`，使用 `auth.DefaultTokenValidator`
- 鉴权客户端：使用 `auth.DefaultTokenTransport`

> 详细指南参见 `references/das-rest-sdk-go-guide.md`

## Detailed References

完整规范文档位于 `references/` 目录，按需查阅：

| 文件 | 内容 |
|------|------|
| `references/das-rest-api-guide.md` | REST API 设计指南全文（URL、方法、状态码、响应、资源集合、JSON、版本、命名） |
| `references/das-openapi-auth.md` | OPENAPI 统一鉴权规范全文（流程、签名、Token API、错误码、示例） |
| `references/das-rest-sdk-go-guide.md` | Go SDK 接入指南全文（Handler、响应、分页排序、鉴权、框架集成） |
| `references/das-rest-sdk-java-guide.md` | Java SDK 接入指南全文（Maven/Gradle、响应构造、分页排序、鉴权、字段常量） |

## Common Mistakes

| 错误 | 正确做法 |
|------|---------|
| URL 使用动词 `/api/agent/list` | 使用复数名词 `/api/v1.0/agents` |
| URL 无版本号 `/api/agents` | 加版本号 `/api/v1.0/agents` |
| 内部接口用 `/openapi` 前缀 | 内部用 `/api`，对外用 `/openapi` |
| 成功响应不含 `data` 键 | 所有成功响应**必须**含 `data` |
| 错误响应不含 `error.code` | **必须**含 `code` + `message` |
| 返回 null 值字段 | String→`""`、Number→`0`、Array→`[]`、Object→`{}` |
| 时间用 ISO 字符串 | **必须**用 Unix 毫秒时间戳 |
| 分页参数用 page/size | **必须**用 `$page`/`$size` |
| 排序参数用 sort/orderBy | **必须**用 `$orderBy` |
| 自定义错误码 | **必须**使用规范定义的 ErrorCode |
| Body 参数参与签名 | **只有 Query 参数参与签名** |
| 版本更新时删除旧属性 | **禁止**删除/修改旧属性，只允许新增 |
| DELETE 响应返回 200 | DELETE 成功应返回 **204 No Content** |
| POST 响应返回 200 | POST 创建成功应返回 **201 Created** |
