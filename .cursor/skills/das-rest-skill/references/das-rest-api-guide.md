# DAS REST API 指南

| 修订日期   | 版次 | 修订人 | 修订内容                                                     |
| ---------- | ---- | ------ | ------------------------------------------------------------ |
| 2023-12-08 | v0.1 | 张德林 | 初稿                                                         |
| 2023-12-22 | v1.0 | 张德林 | URL结构新增对外部接口的定义补充<br />版本发布                |
| 2024-07-04 | v1.1 | 张德林 | 「错误响应」中innererror调整为 ineerError<br />「标准请求头」中增加国际化规范链接 |
|            |      |        |                                                              |



## 1. 摘要

DAS REST API指南作为一种设计规范，鼓励开发同学通过RESTful HTTP接口访问资源。

本文档建立了DAS REST API应该遵循的指导原则，以便开发统一一致的RESTful接口。

## 2. 引言

为了给开发同学提供更流畅的开发体验，让API遵循统一的设计准则很重要，这些准则能让API简单且直观。

API一致性的好处在于使团队拥有统一的代码、模式、文档风格和设计策略。

这些准则旨在达成如下目标：

- 尽可能地遵循行业普遍接受的 REST/HTTP 最佳实践；
- 内部接口层面，让前后端对接更为顺畅；
- 对外接口层面，让第三方开发同学都可以轻松的通过REST接口访问的DAS OPEN API；
- 接口测试层面，让测试同学可以按照标准更容易地去进行自动化接口测试；
- 让其他部门、合作伙伴使用这些准则来设计自己的REST API。

### 2.1 推荐阅读

了解REST架构风格背后的理念，更有助于开发优秀的基于 HTTP 的服务。 如果您对 RESTful 设计不熟悉，请参阅以下优秀资源：

- [RFC7231](https://tools.ietf.org/html/rfc7231) -- 定义HTTP/1.1 语义规范的权威文档
- [REST论文](https://www.ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm) -- Roy Fielding网络架构论文中关于REST的章节，“架构风格与基于网络的软件体系结构设计”
- [理解RESTful架构](http://www.ruanyifeng.com/blog/2011/09/restful.html) -- 一篇介绍RESTful架构较为准确的文章
- [生动理解REST](https://www.zhihu.com/question/27785028) -- 一篇介绍REST较为生动易懂的文章
- [微软API规范](https://github.com/Microsoft/api-guidelines/blob/vNext/Guidelines.md#1444-example-response) -- 本指南的主要参考规范

## 3. 指南解读

### 3.1 应用指南

**这些准则适用于研发中心（或任何公司内其他部门、公司外合作伙伴）开发服务里的公开REST API，即OPENAPI。**私有或内部API也应该尝试遵循这些准则，保证一致性不仅对外部用户有价值，对内部服务使用者也很有价值。这些准则为任何服务都提供了最佳实践。 

如果有合理理由可不遵循这些准则，如：

一些服务与某些外部定义的REST API互操作时必须与某些外部的API兼容，允许其无法遵循这些准则。

一些服务可能具有需要特殊功能上或性能上的需求，允许其无法遵循这些准则。

### 3.2 历史接口

不建议仅仅为了遵从指南而对这些指南之前的旧服务或模块进行重大更改。

### 3.3 “要求”关键字

本文档中的 ”必须”（MUST、REQUIRED、SHALL）、”禁止”（MUST NOT、SHALL NOT）、”推荐”（SHOULD、RECOMMENDED）、 ”可以”（MAY）、”可选“（OPTIONAL） 等关键字的详细解释见 [RFC2119](https://tools.ietf.org/html/rfc2119)。

## 4. 分类

服务必须符合下面定义的分类法。

### 4.1 错误（Error）

错误，定义为因客户端向服务传递错误数据，导致服务端拒绝该请求。客户端传递错误的或者不合法的数据的情况通常返回 “4xx” 的 HTTP错误码。

错误不会影响API的整体可用性。

例如 无效凭证、错误的参数、未知的版本ID等。

### 4.2 故障（Faults）

故障，定义为服务无法正确返回数据以响应有效的客户端请求。通常会返回“5xx”HTTP错误码。

故障会影响整体 API 的可用性。故障意味着服务端代码出现故障，可能会影响整体的API使用。

例如 数据库连接超时。

## 5. 客户端要求

为确保客户端更好的接入REST服务，客户端应遵循以下最佳实践:

### 5.1 字段忽略原则

客户端**必须**安全地忽略未约定的字段。

在产品迭代的过程中，有些服务接口可能在不更改版本号的情况下向响应数据中添加字段。此类服务接口**必须**在其文档中注明，而客户端**必须**可以忽略这些未知字段。

### 5.2 字段排序忽略原则

客户端处理响应数据时**禁止**依赖服务端JSON响应数据字段的顺序。

当服务器返回的 JSON 对象中的字段顺序发生变化，客户端应当能够正确进行解析处理。

服务端**可以**在返回值中显式说明指定某些元素按特定方式进行排序，客户端处理数据时**可以**依赖于服务端明确指定的排序行为。

### 5.3 无声失效原则

当客户端请求带可选功能参数的服务时（包含带可选的头部信息），**必须**对服务端的返回格式有一定兼容性，**可以**忽略这些特定功能。

例如 分页数、排序、国际化等自定义参数的支持和返回格式的兼容。

## 6. 基础原则

### 6.1 URL结构☆

结构约束：

```http
{schema}://{serviceRoot}/{collection}[/{id}]
{schema}://{host}:{port}/{prefix}/{version}/{collection}[/{id}]
```

- schema - API接口支持的协议（http或https）
- {serviceRoot} – 站点URL (site URL) + 服务的根路径的组合
  - host - 服务的域名或IP
  - port - 服务的端口
  - prefix - 接口的标识前缀：内部接口使用"/api"，对外接口使用"/openapi"
  - version - 接口的版本
- {collection} – 集合的名称，复数
- {id} – 唯一标识属性的值，唯一标识中原则上不允许使用”/“，如果业务需要，一定要带”\“转义。

API URL路径结构应该是友好的易于理解的。甚至用户无需通过阅读API文档能够猜出相关结构和路径。

正例：https://das.com/api/v1.0/agents/JHF8UE6H5W34D/memorys

显而易见，这是一个： 版本为v1.0的 获取标识为“JHF8UE6H5W34D”探针内存数据资源 的API。

反例：https://das.com/api/EWS/OData/agents('JHF8UE6H5W34D')/Folders('MEM')

虽然也能理解部分含义，但结构混乱，解析困难且不友好。

URL结尾为**资源（resource）**，既然是资源，必须是名词，必须为复数形式。

反例：

```
GET /api/agent/list?agentType=ainta
GET /api/agent/getAgentById?id=JHF8UE6H5W34D
POST /api/agent/deleteAgentById?id=JHF8UE6H5W34D
```

正例：

```
GET /api/v1.0/agents?agentType=ainta
GET /api/v1.0/agents/JHF8UE6H5W34D
DELETE /api/v1.0/agents/JHF8UE6H5W34D
```

### 6.2 URL长度

HTTP/1.1 标准中并未对请求长度做限制，其中包含URL。
RFC 7230 原文：

> HTTP does not place a predefined limit on the length of a request-line. [...] A server that receives a request-target longer than any URI it wishes to parse MUST respond with a 414 (URI Too Long) status code.

对应翻译：

> HTTP不会对请求行的长度设置预定义的限制。[…]如果服务器接收的请求目标比它希望解析的任何URI都长，则必须使用414（URI过长）状态码进行响应。

此处约定URL长度不超过2,000个字符，约定依据是IE所能接收的URL长度为2083（其他浏览器也有限定但大于这个值）。

不同客户端支持的最长 URL 长度参见以下资料：

- http://stackoverflow.com/a/417184
- https://blogs.msdn.microsoft.com/ieinternals/2014/08/13/url-length-limits/

### 6.3 规范标识符

在URL中，针对具体的资源，必须提供一个唯一且稳定的标识符。

包含规范标识符的URL的一个例子依然是6.1中的正例：

https://das.com/api/v1.0/agents/JHF8UE6H5W34D/memory

标识符推荐使用有含义的唯一性索引字段（IP、邮箱），也可以是暴露给客户端的主键。

### 6.4 操作方法☆

对于资源的具体操作类型，由HTTP Method表示。

| 方法       | 描述                                                         | 是否幂等 |
| ---------- | ------------------------------------------------------------ | -------- |
| **GET**    | 返回资源的当前值<br />成功状态码：200（OK）                  | ✔        |
| **POST**   | 根据提供的数据创建一个新资源，或者提交一个操作<br />成功状态码：201（Created） |          |
| **PUT**    | 更新资源（客户端提供改变后的完整资源，或者称：替换资源）<br />成功状态码：200（OK） | ✔        |
| **PATCH**  | 更新资源（客户端提供改变的属性）<br />成功状态码：200（OK）<br />针对不存在的资源的 PATCH 调用必须使用409（Conflict）响应错误 |          |
| **DELETE** | 删除资源<br />成功状态码：204（No Content）                  | ✔        |
| HEAD       | 返回GET响应的资源的元数据，不常用<br />成功状态码：200（OK） | ✔        |
| OPTIONS    | 获取关于请求的信息，不常用<br />成功状态码：200（OK）        | ✔        |

例子：

```
GET /agents：列出所有探针
POST /agents：新建一个探针
GET /agents/{agentCode}：获取某个指定探针的信息
PUT /agents/{agentCode}：更新某个指定探针的信息（提供该探针的全部信息）
PATCH /agents/{agentCode}：更新某个指定探针的的信息（提供该探针的部分信息）
DELETE /agents/{agentCode}：删除某个指定探针
GET /agents/{agentCode}/monitors：列出某个指定探针的所有监控数据
DELETE /agents/{agentCode}/monitors/{monitorId}：删除某个指定探针的指定监控数据
```

### 6.5 标准请求头

当前有规范的请求头参数如下所示，使用这些请求头参数不是强制性的，但如果使用它们则必须始终一致地遵循 DAS REST API指南服务使用。

| 请求头参数      | 类型                                  | 描述                                                         |
| --------------- | ------------------------------------- | ------------------------------------------------------------ |
| Accept          | Content type                          | 请求的响应内容类型<br />一般为 Accept: application/json      |
| Accept-Language | "en-US", "zh-CN", etc.                | 指定响应的首选语言。<br />产品如支持国际化（i18n），无特殊情况使用[Accept-Language](https://developer.mozilla.org/zh-CN/docs/Web/HTTP/Headers/Accept-Language)来指定语言。<br />国际化请遵循[国际化开发规范](https://scc.das-security.cn/#/docs/specifications?position=DAS-i18n-DEV%2525E8%2525A7%252584%2525E8%25258C%252583%253Fid%253D%2525e5%25259b%2525bd%2525e9%252599%252585%2525e5%25258c%252596%2525e5%2525bc%252580%2525e5%25258f%252591%2525e8%2525a7%252584%2525e8%25258c%252583)。 |
| Accept-Charset  | Charset type like "UTF-8"             | 指定响应的编码。默认值是UTF-8。                              |
| Content-Type    | Content type                          | 请求正文的媒体（Mime）类型 (PUT/POST/PATCH)<br />媒体类型在[RFC2854](https://www.rfc-editor.org/rfc/rfc2854.txt)有明确定义 |
| Prefer          | return=minimal, return=representation | 如果指定了return = minimal首选项，则服务应该返回一个空主体（empty body）以响应一次成功的插入或更新。<br />如果指定了return = representation，则服务应该返回创建或更新的资源。<br />使用场景：客户端有时会需要从响应中获得资源，但有时响应会对带宽造成太大的影响而不想获得资源。 |

### 6.6 标准响应头

服务应该返回以下响应头参数。

| 响应头参数      | 类型                                   | 描述                                                         |
| --------------- | -------------------------------------- | ------------------------------------------------------------ |
| Content-Type    | Content type                           | 请求正文的媒体（Mime）类型 (PUT/POST/PATCH)<br />媒体类型在[RFC2854](https://www.rfc-editor.org/rfc/rfc2854.txt)有明确定义 |
| X-Frame-Options | DENY、SAMEORIGIN、ALLOW-FROM uri、NONE | 指示允许一个页面可否在 \<frame\>,\ </iframe\> 或者 \<object\> 中展现的标记。<br />无特殊需要接口开发不指定。 |

### 6.7 状态码☆

应使用标准HTTP状态码作为响应状态码

HTTP 状态码就是一个三位数，分成五个类别。

- `1xx`：相关信息
- `2xx`：操作成功
- `3xx`：重定向
- `4xx`：客户端错误
- `5xx`：服务器错误

这五大类总共包含[100多种](https://en.wikipedia.org/wiki/List_of_HTTP_status_codes)状态码，覆盖了绝大部分可能遇到的情况。每一种状态码都有标准的（或者约定的）解释，客户端只需查看状态码，就可以判断出发生了什么情况，所以服务器应该返回尽可能精确的状态码。

API 不需要`1xx`状态码，下面介绍其他四类状态码的精确含义。

#### 6.7.1 常用状态码☆

以下列举了常用的状态码，表述其使用场景及对应使用说明。

##### 2xx状态码

`200`状态码表示操作成功，但是不同的方法可以返回更精确的状态码。

> - GET: 200 OK
> - POST: 201 Created
> - PUT: 200 OK
> - PATCH: 200 OK
> - DELETE: 204 No Content

上面代码中，`POST`返回`201`状态码，表示生成了新的资源；`DELETE`返回`204`状态码，表示资源已经不存在。

此外，`202 Accepted`状态码表示服务器已经收到请求，但还未进行处理，会在未来再处理，通常用于异步操作。
下面是一个例子。

 ```http
 HTTP/1.1 202 Accepted
 {
   "task": {
     "href": "/api/update-jobs/12345",
     "id": "12345"
   }
 }
 ```

##### 4xx 状态码

`4xx`状态码表示客户端错误，主要有下面几种。

`400 Bad Request`：服务器不理解客户端的请求，未做任何处理。

`401 Unauthorized`：用户未提供身份验证凭据，或者没有通过身份验证。

`403 Forbidden`：用户通过了身份验证，但是不具有访问资源所需的权限。

`404 Not Found`：所请求的资源不存在，或不可用。

`405 Method Not Allowed`：用户已经通过身份验证，但是所用的 HTTP 方法不在他的权限之内。

`415 Unsupported Media Type`：客户端要求的返回格式不支持。比如，API 只能返回 JSON 格式，但是客户端要求返回 XML 格式。

`422 Unprocessable Entity` ：客户端上传的附件无法处理，导致请求失败。

`429 Too Many Requests`：客户端的请求次数超过限额。

##### 5xx 状态码

`5xx`状态码表示服务端错误。一般来说，API 不会向用户透露服务器的详细信息，所以只要两个状态码就够了。

`500 Internal Server Error`：客户端请求有效，服务器处理时发生了意外。

`503 Service Unavailable`：服务器无法处理请求，一般用于网站维护状态。

#### 6.7.2 状态码表

| 状态码 | 状态码英文名称                  | 中文描述                                                     |
| :----- | :------------------------------ | :----------------------------------------------------------- |
| 100    | Continue                        | 继续。[客户端](http://www.dreamdu.com/webbuild/client_vs_server/)应继续其请求 |
| 101    | Switching Protocols             | 切换协议。服务器根据客户端的请求切换协议。只能切换到更高级的协议，例如，切换到HTTP的新版本协议 |
|        |                                 |                                                              |
| 200    | OK                              | 请求成功。一般用于GET与POST请求                              |
| 201    | Created                         | 已创建。成功请求并创建了新的资源                             |
| 202    | Accepted                        | 已接受。已经接受请求，但未处理完成                           |
| 203    | Non-Authoritative Information   | 非授权信息。请求成功。但返回的meta信息不在原始的服务器，而是一个副本 |
| 204    | No Content                      | 无内容。服务器成功处理，但未返回内容。在未更新网页的情况下，可确保浏览器继续显示当前文档 |
| 205    | Reset Content                   | 重置内容。服务器处理成功，用户终端（例如：浏览器）应重置文档视图。可通过此返回码清除浏览器的表单域 |
| 206    | Partial Content                 | 部分内容。服务器成功处理了部分GET请求                        |
|        |                                 |                                                              |
| 300    | Multiple Choices                | 多种选择。请求的资源可包括多个位置，相应可返回一个资源特征与地址的列表用于用户终端（例如：浏览器）选择 |
| 301    | Moved Permanently               | 永久移动。请求的资源已被永久的移动到新URI，返回信息会包括新的URI，浏览器会自动定向到新URI。今后任何新的请求都应使用新的URI代替 |
| 302    | Found                           | 临时移动。与301类似。但资源只是临时被移动。客户端应继续使用原有URI |
| 303    | See Other                       | 查看其它地址。与301类似。使用GET和POST请求查看               |
| 304    | Not Modified                    | 未修改。所请求的资源未修改，服务器返回此状态码时，不会返回任何资源。客户端通常会缓存访问过的资源，通过提供一个头信息指出客户端希望只返回在指定日期之后修改的资源 |
| 305    | Use Proxy                       | 使用代理。所请求的资源必须通过代理访问                       |
| 306    | Unused                          | 已经被废弃的HTTP状态码                                       |
| 307    | Temporary Redirect              | 临时重定向。与302类似。使用GET请求重定向                     |
|        |                                 |                                                              |
| 400    | Bad Request                     | 客户端请求的语法错误，服务器无法理解                         |
| 401    | Unauthorized                    | 用户未提供身份验证凭据，或者没有通过身份验证                 |
| 402    | Payment Required                | 保留，将来使用                                               |
| 403    | Forbidden                       | 用户通过了身份验证，但是不具有访问资源所需的权限             |
| 404    | Not Found                       | 所请求的资源不存在，或不可用                                 |
| 405    | Method Not Allowed              | 用户已经通过身份验证，但是所用的 HTTP 方法不在他的权限之内   |
| 406    | Not Acceptable                  | 服务器无法根据客户端请求的内容特性完成请求                   |
| 407    | Proxy Authentication Required   | 请求要求代理的身份认证，与401类似，但请求者应当使用代理进行授权 |
| 408    | Request Time-out                | 服务器等待客户端发送的请求时间过长，超时                     |
| 409    | Conflict                        | 服务器完成客户端的 PUT 请求时可能返回此代码，服务器处理请求时发生了冲突 |
| 410    | Gone                            | 客户端请求的资源已经不存在。410不同于404，如果资源以前有现在被永久删除了可使用410代码，网站设计人员可通过301代码指定资源的新位置 |
| 411    | Length Required                 | 服务器无法处理客户端发送的不带Content-Length的请求信息       |
| 412    | Precondition Failed             | 客户端请求信息的先决条件错误                                 |
| 413    | Request Entity Too Large        | 由于请求的实体过大，服务器无法处理，因此拒绝请求。为防止客户端的连续请求，服务器可能会关闭连接。如果只是服务器暂时无法处理，则会包含一个Retry-After的响应信息 |
| 414    | Request-URI Too Large           | 请求的URI过长（URI通常为网址），服务器无法处理               |
| 415    | Unsupported Media Type          | 服务器无法处理请求附带的媒体格式                             |
| 416    | Requested range not satisfiable | 客户端请求的范围无效                                         |
| 417    | Expectation Failed              | 服务器无法满足Expect的请求头信息                             |
|        |                                 |                                                              |
| 500    | Internal Server Error           | 服务器内部错误，无法完成请求                                 |
| 501    | Not Implemented                 | 服务器不支持请求的功能，无法完成请求                         |
| 502    | Bad Gateway                     | 作为网关或者代理工作的服务器尝试执行请求时，从远程服务器接收到了一个无效的响应 |
| 503    | Service Unavailable             | 由于超载或系统维护，服务器暂时的无法处理客户端的请求。延时的长度可包含在服务器的Retry-After头信息中 |
| 504    | Gateway Time-out                | 充当网关或代理的服务器，未及时从远端服务器获取请求           |
| 505    | HTTP Version not supported      | 服务器不支持请求的HTTP协议的版本，无法完成处理               |

更多状态码详情，参见：[RFC2616-sec10](https://www.w3.org/Protocols/rfc2616/rfc2616-sec10.html)



### 6.8 响应格式☆

此章节为重点，此处规定了可读性较好并且一致的响应结果，开发人员使用 HTTP工具方法（或SDK）处理响应（TODO）。

没有特殊需求，推荐使用JSON作为传输格式。JSON属性名应该采用驼峰命名规范 ([RFC3864](https://tools.ietf.org/html/rfc3864))。

#### 6.8.1 响应格式指定

客户端应该使用Accept参数请求响应格式。服务端可以选择性地忽略，如客户端发送多个Accept参数值，服务可以选择其中一个格式进行响应。

默认的响应格式（没有指定Accept）应该是application/json。

`Accept: application/json`

| 请求参数类型     | 响应类型           |
| ---------------- | ------------------ |
| application/json | 必须是返回json格式 |

#### 6.8.2 标准响应规范☆

对于调用成功的情况，规定统一的成功响应的字段规范。

##### SuccessResponse

对象类型

| 字段   | 类型                 | 是否必填 | 描述                                       |
| ------ | -------------------- | -------- | ------------------------------------------ |
| `data` | Object、Object[] ... | ✔        | 一般为操作的资源，也可以是客户端约定的标识 |
| ...    | ...                  |          | 其他业务需要的拓展字段                     |

成功响应**必须**是单个JSON对象。为保证一致性，该对象必须有一个名为“data”的键，其值类型按不同情况返回。

如果是单个资源的获取，“data”必须是单个JSON对象。

如果是多个资源的获取，“data”必须是包含0-n个对象的JSON数组。

##### 成功响应样例

例子「单个对象」：

```http
GET /api/agent-configs/soc
{
  "data": {
    "agentType":"soc",
    "softVersion":"3.0.1",
    "config":"",
    "version":1611054006000
  }
}
```

例子「多个对象」

```http
GET /api/agent-configs
{
  "data": [
    {
    "agentType":"soc",
    "softVersion":"3.0.1",
    "config":"",
    "version":1611054006000
  	},
    {
    "agentType":"ainta",
    "softVersion":"1.1.3",
    "config":"",
    "version":1611054006000
  	}]
}
```

```http
GET /api/agent-configs
{
  "data": []
}
```



例子「成功执行」

```http
DELETE /api/agents/HI6J5OEJ23FKH83W
{
  "data":1
}
```



#### 6.8.3 错误响应规范☆

对于调用不成功的情况，规定统一的错误响应的字段规范。

##### ErrorResponse

对象类型

| 字段    | 类型  | 是否必填 | 描述                   |
| ------- | ----- | -------- | ---------------------- |
| `error` | Error | ✔        | 错误对象               |
| ...     | ...   |          | 其他业务需要的拓展字段 |

##### Error

对象类型

| 字段         | 类型       | 是否必填 | 描述                                                       |
| ------------ | ---------- | -------- | ---------------------------------------------------------- |
| `code`       | String     | ✔        | 服务器定义的固定错误代码，禁止随意定义，必须跟随版本发布。 |
| `message`    | String     | ✔        | **必须**是可读且易于理解，可作为面向用户的提示消息。       |
| `target`     | String     |          | 具体的目标。                                               |
| `details`    | Error[]    |          | 更细分的多个错误用错误组表示，Error对象数组类型。          |
| `innerError` | InnerError |          | 包含比当前对象更具体的错误信息的对象，InnerError对象类型。 |

错误响应**必须**是单个JSON对象。该对象必须有一个名为“error”的键，其值**必须**是JSON对象。

error对象**必须**包含名称“code”和“message”的键值对，并且**推荐**包含譬如“target”、“details”和 “innerError” 的键值对。

“code”值是该服端务定义的错误代码，应该简单可读。与响应中指定的HTTP错误代码相比，此代码用作定位更具体的错误。

##### InnerError

对象类型

| Property     | Type       | Required | Description                                                  |
| ------------ | ---------- | -------- | ------------------------------------------------------------ |
| `code`       | String     | ✔        | 比包含错误的提供的更具体的错误代码，允许自行拓展，区别于Error对象中的code。 |
| `innerError` | InnerError |          | 包含比当前对象更具体的错误信息的对象，InnerError对象类型。<br />最重要的是支持嵌套。 |

如果要指明更为具体的错误，**应该**在“innerError” 对象中定义。

“message” 键值对的值**必须**是错误提示消息，**必须**是可读且易于理解，可作为面向用户的提示消息。

“target” 键值对的值 是指向错误的具体的目标（例如，错误中属性的名称）。

“details”键值对的值 必须是JSON对象数组，数组中的对象通常表示请求期间发生的不同的、相关的错误。

##### Error.code

通用的错误code（待完善）

| 错误代码                | 响应码 | 描述                                                         |
| :---------------------- | ------ | :----------------------------------------------------------- |
| BadArgument             | 400    | 提供的参数无效                                               |
| BadUserArgument         | 400    | 提供的用户参数无效                                           |
| ErrorUnsupportedOrderBy | 400    | 资源不支持排序 或 不支持使用指定的字段排序                   |
| ErrorUnsupportedPaging  | 400    | 资源不支持分页请求                                           |
| InvalidOperation        | 400    | 请求在当前上下文中无效<br />例子：<br />JOB提交中，不可修改。<br />模型与请求版本不兼容无法更新。 |
| Unauthorized            | 401    | 请求未获授权，无法访问资源。                                 |
| InvalidApiKey           | 401    | 无效apiKey。                                                 |
| NotFound                | 404    | 找不到资源。                                                 |
| RequestTimeout          | 408    | 无法在允许的时间内完成该操作。                               |
| Conflict                | 409    | 请求资源冲突，POST已存在资源 或 PATCH不存在资源。            |
| Overlimit               | 409    | 请求资源超出系统限制。                                       |
| InternalError           | 500    | 执行时遇到内部错误。<br />具体异常类型自定义，放入innerError |
| ServiceUnavailable      | 503    | 服务暂时不可用。                                             |
| GatewayTimeout          | 504    | 无法在允许的时间内完成该操作。                               |

##### 错误响应样例

例子「带有”innerError“」

```json
{
  "error": {
    "code": "BadArgument",
    "message": "Previous passwords may not be reused",
    "target": "password",
    "innerError": {
      "code": "PasswordError",
      "innerError": {
        "code": "PasswordDoesNotMeetPolicy",
        "minLength": "6",
        "maxLength": "64",
        "characterTypes": ["lowerCase","upperCase","number","symbol"],
        "minDistinctCharacterTypes": "2",
        "innerError": {
          "code": "PasswordReuseNotAllowed"
        }
      }
    }
  }
}
```

在本例中，基本的错误码是“BadArgument”，而“innerError”中提供了更具体的错误代码。 “PasswordReuseNotAllowed”代码可能是在之后的迭代中新增的，之前只返回“PasswordDoesNotMeetPolicy”。 这种增量型的添加方式并不会破坏老的客户端的处理过程，而又可以给开发者一些更详细的信息。

例子「带有“details”」

```json
{
  "error": {
    "code": "BadArgument",
    "message": "Multiple errors in ContactInfo data",
    "target": "ContactInfo",
    "details": [
      {
        "code": "NullValue",
        "target": "PhoneNumber",
        "message": "Phone number must not be null"
      },
      {
        "code": "NullValue",
        "target": "LastName",
        "message": "Last name must not be null"
      },
      {
        "code": "MalformedValue",
        "target": "Address",
        "message": "Address is not valid"
      }
    ]
  }
}
```

在本例中，请求存在多处问题，每个错误都列在 "details" 字段中进行返回了。

#### 6.8.4 不支持的请求

客户端可以请求当前不受支持的功能。 但服务端必须响应与本节一致的有效但不受支持的请求。

如果服务不支持一些功能，则必须在客户端请求API时返回错误响应。 

举例：

- 用key在集合中检索，例如：`https://das.com/api/v1.0/agents/JHF8UE6H5W34D`
- 用条件过滤集合，例如：`https://das.com/api/v1.0/agents?agentType=soc`
- 通过_$page_和_$size_进行分页，例如：`https://das.com/api/v1.0/agents?$page=5&$size=10`
- 按_$orderBy_排序，例如：`https://das.com/api/v1.0/agents?$orderBy=createTime desc`

错误响应**必须**是来自4xx系列的HTTP状态码，表示无法满足请求。

除非更具体的错误状态适用于给定的请求，否则服务应该返回“400 Bad Request”。服务应该在响应消息中包含足够的详细信息，以便开发人员明确不支持请求的哪个部分。

例子：

```http
GET https://das.com/api/v1.0/agents?$orderBy=name HTTP/1.1
Accept: application/json
```

```http
HTTP/1.1 400 Bad Request
Content-Type: application/json
{
  "error": {
    "code": "ErrorUnsupportedOrderBy",
    "message": "Ordering by name is not supported."
  }
}
```

## 7. 资源集合

### 7.1 资源标识

资源集合中每个具体资源推荐使用持久标识符（主键），含义等同于6.3章节的定义。

### 7.2 序列化

使用JSON作为序列化标准格式。

### 7.3 集合的URL

URL中集合直接定义在根路径之后，或者在另一个具体资源之后。

集合元素本质是 **资源（resource）**，既然是资源，必须是名词，必须为复数形式。

```http
GET https://{serviceRoot}/{collection}/{id}
```

- {serviceRoot} – 站点URL (site URL) + 服务的根路径的组合
- {collection} – 集合的名称，复数
- {id} – 唯一标识属性的值，唯一标识中原则上不允许使用”/“，如果业务需要，一定要带”\“转义。

例子：

**常规资源集合**

```http
GET https://das.com/api/v1.0/agents
```

**嵌套资源集合和属性**

集合项可以包含其他集合。例如，探针集合可能包含多个内存数据资源：

```http
GET https://das.com/api/v1.0/agents/JHF8UE6H5W34D/memorys
```

### 7.3 通用子资源

资源集合根据业务场景及具体数据定义，但部分子资源对多数资源来说通用的，此处定义的通用子资源如涉及，每个资源定义必须遵守。

#### 7.4.1 files（文件子资源）

*files*表述的是一个资源集合或具体资源的文件子资源，常用于 导入、导出 场景。

使用可选过滤参数*type*来区分不同文件类型，服务端**必须**定义默认type，当未指定type时使用默认文件类型。服务端也**应该**在只支持一种默认type的时候，可以忽略客户端发来的type参数。

导出或下载使用GET，表述意义为获取资源的文件子资源。

导入或上传使用POST，表述意义为创建资源的文件子资源。

例子：

**获取资源集合的文件子资源（导出资源集合）**

```http
GET https://das.com/api/v1.0/agents/files
Accept: application/json
HTTP/1.1 200 OK
Content-Type: application/octet-stream
```

**获取特定资源的文件子资源（导出特定资源）**

```http
GET https://das.com/api/v1.0/agents/JHF8UE6H5W34D/files?type=docx
Accept: application/json
HTTP/1.1 200 OK
Content-Type: application/octet-stream
```

**创建资源集合的文件子资源（导入资源集合）**

```http
POST https://das.com/api/v1.0/agents/files
```

**创建特定资源的文件子资源（导入特定资源）**

```http
POST https://das.com/api/v1.0/agents/JHF8UE6H5W34D/files?type=docx
```



#### 7.4.2 synchronizations（同步子资源）

*synchronizations*表述的是从一个资源集合同步全部或部分数据到另一个资源集合。资源集合可以是内部的也可以是外部的。

一般情况下，创建同步子资源是异步的，响应码应该返回202。

例子：

 ```http
 POST https://das.com/api/v1.0/assets/synchronizations
 Accept: application/json
 {
  	"from":"https://soc.com/api/assets"
 }
 
 HTTP/1.1 202 Accepted
 {
   "task": {
     "href": "/api/v1.0/assets/synchronizations/12345",
     "id": "12345"
   }
 }
 ```



#### 7.4.3 connectivities（连通性子资源）

*connectivities*表述的是一个资源集合或具体资源的连通性子资源，常用于检测资源（通常是设备类资源）的连通性。

连通性的检测包含几类：系统 -> 资源，资源 -> 资源，资源 -> 外部实体。

例子：

**创建特定资源的连通性子资源（检测特定资源的连通性 - 常用）**

- 测试系统对特定资产连通性。系统 -> 资源。

```http
 POST https://das.com/api/v1.0/assets/JHF8UE6H5W34D/connectivities
```

- 测试邮件服务资源对外部邮箱的连通性。资源 -> 外部实体。

```http
 POST https://das.com/api/v1.0/email-servers/JHF8UE6H5W34D/connectivities
 {
 		"to":"caphub@das.com"
 }
```

**创建资源集合的连通性子资源（检测资源集合的连通性）**

```http
POST https://das.com/api/v1.0/assets/connectivities
```



#### 7.4.4 restorations（重置子资源）

*restorations*表述的是重置资源集合到另一个资源集合，用于资源（通常是配置类资源）的重置。资源集合可以是内部的也可以是外部的。

可以是重置到原始默认（default），可以是重置到系统配置（system），可以是重置到某个版本的(versionId)，亦或是其他合理可以找到资源集合的标识。

例子：

 ```http
POST https://das.com/api/v1.0/menus/restorations
Accept: application/json
{
 	"from":"default、system、v2_1"
}
 ```



#### 7.4.5 options（选项子资源）

*options*表述的是选项子资源，一般仅需包含资源的标识字段和命名字段，用于给资源使用方作为选项使用。

目的：

1. 主要目的是减少冗余字段的返回；
2. 便于资源使用封装统一的组件或方法。

支持两种字段规范：显式、映射式

**显式**

存在两个规范字段：label、value

| Property | Type   | Required | Description          |
| -------- | ------ | -------- | -------------------- |
| `label`  | String | ✔        | 用于展示的名称       |
| `value`  | -      | ✔        | 用于提交的标识或内容 |
| ...      |        |          | 其他字段             |

例子：

```http
GET https://das.com/api/v1.0/menus/options
Accept: application/json
[
  {
    "label":"首页",
    "value":"HomePage"
  },
  {
    "label":"检索",
    "value":"Search"
  }
]
```



**映射式**

除返回的数据集data外，存在mapping字段，mapping存在两个映射字段：label、value，用于定义映射关系。

| Property  | Type   | Required | Description              |
| --------- | ------ | -------- | ------------------------ |
| `data`    | Array  | ✔        | 资源集合（剔除无关字段） |
| `mapping` | Object | ✔        | 映射对象                 |
| `» label` | String | ✔        | Label映射字段            |
| `» value` | String | ✔        | Value映射字段            |

例子：

```http
GET https://das.com/api/v1.0/menus/options
Accept: application/json
{
    "data":
    [
        {
            "key1": "首页",
            "key2": "HomePage",
            "key3": "xxx"
        },
        {
            "key1": "检索",
            "key2": "Search"
        }
    ],
    "mapping":
    {
        "label": "key1",
        "value": "key2"
    }
}
```





### 7.5 排序

*$orderBy* 参数允许客户端对集合查询的结果进行排序。

*$orderBy* 参数的值包含可以是多个，用逗号分隔的。 可以指定排序方式（asc 升序、desc 降序），空格分隔，默认为升序。

空值（NULL）排序必须“小于”非空值。

如果不支持排序字段，需统一返回 400 Bad Request，错误码：ErrorUnsupportedOrderBy。

例子：

```http
GET https://das.com/api/v1.0/agents?$orderBy=name
```
按名称正序排。

```http
GET https://das.com/api/v1.0/agents?$orderBy=name desc
```
按名称倒序排。

```http
GET https://das.com/api/v1.0/agents?$orderBy=name desc,createTime
```
按名称倒序排，再按创建时间排。

响应内容中，资源必须放在data内，同时**必须**返回*$orderBy*字段。

响应举例：

```http
GET https://das.com/api/v1.0/agents?$orderBy=name desc,createTime
Accept: application/json
HTTP/1.1 200 OK
Content-Type: application/json
{
  ...,
  "$orderBy": "name desc,createTime",
  "data": [...],
  "total":100
}
```

### 7.6 分页

客户端可以使用 *$page*和*$size*查询参数来指定页数和每页的数量。

 *$page*和 *$size*参数的值是正整数（1-N）。

如果不支持分页字段，需统一返回 400 Bad Request，错误码：ErrorUnsupportedPaging。

响应内容中，资源必须放在data内，同时**必须**返回*$page*、*$size*字段，**按需**返回total字段。

例子：

```http
GET https://das.com/api/v1.0/agents?$page=5&$size=10 HTTP/1.1
Accept: application/json
HTTP/1.1 200 OK
Content-Type: application/json
{
  ...,
  "$page": 5,
  "$size": 10,
  "data": [...],
  "total":100
}
```

### 7.7 批量操作

对 资源的批量操作 准确的理解应该是 对资源集合的操作。以下几个例子来助于理解。

例子：

批量获取

```http
GET https://das.com/api/v1.0/agents
```

批量新增

```http
POST https://das.com/api/v1.0/agents
{
	"data":[]
}
```

批量更新（提供改变的资源属性）

```http
PATCH https://das.com/api/v1.0/agents
{
	"data":[]
}
```

批量替换（提供改变后的完整资源）

```http
PUT https://das.com/api/v1.0/agents
{
	"data":[]
}
```

批量删除

```http
# 根据IDs删除
DELETE https://das.com/api/v1.0/agents/JHF8UE6H5W34D,GDSRUEJHFR87K,HJHAS45SDAOL
# 根据条件删除
DELETE https://das.com/api/v1.0/agents?agentType="ainta"
```



## 8. JSON标准化

### 8.1 基本类型

必须按照[RFC8259](https://www.rfc-editor.org/rfc/rfc8259.txt)的规则将原始值序列化为JSON。JSON文本必须使用UTF-8 [RFC3629](https://www.rfc-editor.org/rfc/rfc3629.txt) 编码。

JSON值必须是字符串、数字、布尔值、对象、数组、null，取值**禁止**超过这个范畴。

| **数据类型** | **描述**                                                     | **示例**                |
| ------------ | ------------------------------------------------------------ | ----------------------- |
| **String**   | 字符串的表示类似于C系列编程语言中使用的约定。**字符串以双引号开头和结尾**。所有Unicode字符都可以放在引号中，但必须转义的字符除外：引号、反斜线分隔符\和控制字符(U+0000到U+001 F)。 | "hello"  "\u4f60\u597d" |
| **Number**   | 数字的表示与大多数编程语言中使用的表示类似。**一个数字用十进制数字以基数10表示**。它包含一个整数分量，其前缀可以是可选减号，后面可以是分数部分和/或指数部分。不允许前导零。 | 1234  123.22            |
| **Boolean**  | **值为 true 或 false**  。                                   | true  false             |
| **Object**   | **对象结构表示为围绕零或多个名称/值对(或多个成员)的一对花括号。名称是字符串。在每个名称之后加上一个冒号，将名称与值分隔开来。一个逗号将值与接下来的名称分隔开来。对象中的名称应该是唯一的**。 | {"a":1,  "b":true}      |
| **Array**    | **数组结构表示为围绕零个或多个值(或多个元素)的方括号。元素用逗号分隔**。<br />不要求数组中的值具有相同的类型。 | [1,2,3,4]               |
| **Null**     | 空。                                                         | null                    |

注意：

**64位整数的重要说明：** JavaScript将静默截断大于“Number.MAX_SAFE_INTEGER”（2 ^ 53-1）的整数或小于“Number.MIN_SAFE_INTEGER”（-2 ^ 53 + 1）的数字。 **如果数值类型值过大，必须将该值作为字符串返回**。

### 8.2 空值

为了保证接口的一致性，对空值做如下约束：

|**数据类型** | 空值约束                                                     |
| ------------ | ------------------------------------------------------------ |
|**String**   | ""                                                           |
|**Number**   | 0                                                            |
|**Boolean**  | 不允许存在空值 |
|**Object**   | {} |
|**Array**    | [] |



### 8.3 时间类型

时间字段在各个接口中的定义都不要显得混乱且不统一，**必须**按本节一致的约定执行。

#### 8.3.1 日期\时间约定

通常情况下，对于日期\时间在不同的国家、不同的业务场景下会有相当多种类的不同表示。

为保证接口的一致性，对日期\时间做如下约束：

- 接口层面**必须**使用Unix时间戳传递日期\时间，**必须**毫秒级。例如：1611469304000。
- 建议后端时间存储也采用Unix时间戳存储。
- 前端根据用户的时区、业务场景自行处理呈现。

#### 8.3.2 持续时长

按照[ISO 8601] [wikipedia-iso8601-durations](http://en.wikipedia.org/wiki/ISO_8601#Durations)进行序列化。

持续时间“由格式'P [n] Y [n] M [n] DT [n] H [n] M [n] S`表示。”

- P是在持续时间表示开始时放置的持续时间指示符（“period”）。
- Y是年份指示符，它遵循年数值。
- M是月份指示符，它遵循月数值。
- W是周指示符，它遵循周数值。
- D是遵循天数值的日期指示符。
- T是在表示的时间分量之前的时间指示符。
- H是小时指示符，它遵循小时数值。
- M是遵循分钟数值的分钟指示符。
- S是秒指示符，它遵循秒数的值。

例如，

“P3Y6M4DT12H30M5S”表示“：”三年，六个月，四天，十二个小时，三十分钟加五秒” 的持续时间。



## 9. 版本号

**所有符合DAS REST API准则的API都必须支持显式版本控制。 **

### 9.1 版本定义

使用major.minor格式进行版本定义。

版本定义在/api路径之后。

```http
https://das.com/api/v1.0/agents
```

### 9.2 版本更新说明

**必须**保留历史版本接口，除非你能确认该接口已无任何客户端在使用。

不更新版本的情况下，允许修改接口返回，但只允许新增属性，**禁止**删除和修改原属性（参见5.1章节「字段忽略原则」）。

## 10. 命名准则

### 10.1 建议

命名准则有助于开发同学避免经常查阅资料思考如何去命名字段，可以花更多时间专注研发。使用通用和标准的约定可以帮助开发同学正确猜测到公共属性名称及其含义。 应该使用详细的单词命名，并且不应该使用非通用的缩写词，通用缩写词指的是API范畴内大家公认的(比如Url)。

### 10.2 格式

- 缩写**应该**遵循大小写惯例，就像它们是普通单词一样（例如Url）。
- 所有标识符包括 名称空间（namespaces）、实体类型（entityTypes）、实体集合（entitySets）、属性（properties）、动作（actions）、函数（functions）和枚举值（enumeration）**应该**使用驼峰法（lowerCamelCase）。
- HTTP头属性是例外，其**应该**使用Capitalized-Hyphenated-Terms的标准HTTP约定命名。

### 10.3 避免命名

某些专有词汇的命名应该避免使用，在某些API使用场景里可能会导致冲突。

接口禁止使用以下单词命名：

- Context
- Scope
- Resource

### 10.4 复合名称

- **应该**避免使用诸如'a'，'the'，'of'之类的单词，除非需要有业务必要。
  - 例如 不应该使用诸如 a User，theAccount，countOfBooks 之类的名称，而应该首选user，account，bookCount。
- **应该**向属性名添加类型，如果不这样做会导致数据的表示方式不明确。
  - 向属性名添加类型时，**必须**在末尾添加类型，例如createdDateTime。

### 10.5 标识字段

- **必须**使用字符串类型定义标识字段。
- **可以**使用简单的“id”来表示资源主键值。
- **应该**使用后缀为“Id”的关系名称来表示另一个资源的外键，例如：subscriptionId。
- 此字段的内容应该是引用资源的规范ID。

### 10.6 时间字段

- 对于同时需要日期和时间的字段，**必须**使用后缀“DateTime”，例如：createDateTime。
- 对于只需要日期信息而无需时间的字段，**必须**使用后缀“date”，例如：birthDate。
- 对于只需要时间信息而无需日期的字段，**必须**使用后缀“time”，例如：startTime。

### 10.7 名称属性

- 对于通用命名字段，**必须**使用后缀”Name“，例如：agentName。

### 10.8 集合和计数

- 集合命名必须为复数名词或复数名词短语。
- **可以**使用简化英语来表示不常用的复数名词。
  - 例如**可以**用schemas来替代schemata。
- **必须**使用后缀为“Count”的名词或名词短语来命名资源的计数。

### 10.9 常用字段名称

如果服务有属性的含义跟下面的命名匹配，则服务必须使用此表中的命名。此表将随着服务添加更常用的术语而更新。所有有相关建议的同学都**可以**一起维护这个表格。

| 常用字段列表 |
| -------------------- |
| contentUrl   |
| country      |
| createdBy    |
| createTime   |
| das          |
| displayName  |
| endTime      |
| errorUrl     |
| event        |
| id           |
| jobTitle     |
| location     |
| memberOf     |
| message      |
| name         |
| owner        |
| people       |
| person       |
| picture      |
| postalCode   |
| properties   |
| signInName   |
| startTime    |
| tags         |
| updateTime   |
| webUrl       |
|              |
