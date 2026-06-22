# 开放接口（OPENAPI）统一鉴权规范

| 修订日期   | 版次 | 修订人 | 修订内容 |
| ---------- | ---- | ------ | -------- |
| 2024-07-03 | v1.0 | 谭立聪 | 初版     |
|            |      |        |          |

## 1. 引言

本规范提出一种安全且规范的OPENAPI统一鉴权规则，OPENAPI提供方**必须**按以下规则对OPENAPI进行鉴权，调用方**必须**按以下规则调用OPENAPI。

## 2. 意义

统一的OPENAPI鉴权规则具有重要的战略和技术价值，其意义可以从产品层面和角色层面两个维度进行深入分析。

### 1.1 产品层面的意义

**提升安全性和标准化**

统一的鉴权规则能够显著提升系统的整体安全水平，减少因鉴权机制不一导致的安全漏洞。标准化的规则使得所有接口遵循相同的认证与授权流程，降低了被攻击的风险，同时便于实施统一的安全策略和监控。

**简化内部调用逻辑**

对于内部产品间的服务调用，统一鉴权规则减少了不必要的定制化工作，使得服务间交互更加高效、简洁。开发人员无需为每个服务单独设计鉴权逻辑，降低了维护成本和出错概率。

**促进外部合作与集成**

对外部产品或服务而言，统一的鉴权规则降低了接入安恒产品的门槛，提高了合作效率。合作伙伴和第三方开发者可以快速理解并实现对接，促进了生态系统的扩展和业务的快速发展。

#### 1.2 角色层面的意义

**为开发人员提供清晰指引**

统一的规则为开发团队提供了明确的设计和实现指南，减少了在鉴权逻辑上的摸索和重复工作，使他们能更专注于业务逻辑的实现。

**减轻测试人员工作量**

对于测试团队，统一的鉴权规则使得测试用例的编写更加规范化，易于构建自动化测试框架，确保接口的安全性得到全面覆盖和验证，提高了测试效率和质量。

**增强合作伙伴与客户的体验**

对于合作伙伴和外部客户，统一的鉴权流程简化了他们的集成工作，降低了学习成本。清晰的文档和标准化的API调用方式提升了用户体验，增强了信任感，有利于建立长期的合作关系。

## 2. 流程

1. 调用OPENAPI所需的 appId 和 appSecret 由提供方分发
2. 调用方携带Query参数 appId 及Header参数 timestamp, sign 调用OPENAPI提供方的Token-OPENAPI接口获取 token，其中：
    $sign = Hash(timestamp + appId + appSecret)$​
3. 调用方携带Header参数timestamp, token, nonce, sign 请求OPNEAPI，其中：
    $sign = Hash(paramPairStr + \& + timestamp + token + nonce)$​​​

**参数说明**

| 参数         | 说明                                                         | 作用                                                         |
| ------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| appId        | 调用方的唯一标识，生成token时需要携带；生成方式与分发方式由提供方自行定义，**建议**使用8-16位字母或数字组成的字符串 | 提供方可以将appId与用户、角色、资源等权限进行绑定，对OPENAPI调用方进行细粒度的鉴权 |
| appSecret    | appId绑定的密钥，**禁止**外泄，与appId成对使用，用于获取token时生成请求签名（sign）；生成方式与分发方式由提供方自行定义，**建议**使用32位字母或数字组成的字符串（UUID去除"-"符号） | -                                                            |
| token        | 调用OPENAPI提供方的Token-OPENAPI接口得到的认证参数           | 提供方需要通过token得到appId，进而对OPENAPI进行鉴权          |
| sign         | 请求签名，调用方请求OPENAPI时需要实时计算并携带 sign 参数    | 防止请求参数被篡改                                           |
| timestamp    | 请求发起时的Unix时间戳，精确至毫秒                           | 请求超时校验                                                 |
| paramPairStr | 计算请求签名（sign）的中间结果，为OPENAPI请求中非空Query参数按照参数名ASCII码从小到大排序（字典序），再按照URL键值对的格式（即key1=value1&key2=value2…）拼接成的字符串 | -                                                            |
| nonce        | 长度为 6 的随机字符串，**建议**每次请求OPENAPI前随机生成 nonce 而不是使用固定的 6 位字符串 | 保证请求签名不可预测，防止请求重放                           |

### 2.1 Token-OPENAPI接口定义

OPNEAPI提供方**必须**实现Token-OPENAPI，调用方使用 appId 和 appSecret 调用该接口获取OPENAPI鉴权所需的 token

**GET /openapi/v1.0/app-tokens**

**请求参数**

| 名称      | 位置   | 类型   | 必选 | 说明                               |
| --------- | ------ | ------ | ---- | ---------------------------------- |
| appId     | query  | string | 是   | 调用方的唯一标识                   |
| sign      | header | string | 是   | 请求签名，根据对应签名规则生成     |
| timestamp | header | number | 是   | 请求发起时的Unix时间戳，精确至毫秒 |

> 返回示例

> 成功

```json
{
  "data": {
    "token": "7e09f5d9-84e6-4054-9788-c275e230f51f",
    "expireTime": 1697714277794
  }
}
```

> 请求有误

```json
{
  "error": {
    "code": "BadArgument",
    "message": "The request parameter is invalid."
  }
}
```

> 记录不存在

```json
{
  "error": {
    "code": "NotFound",
    "message": "The appId is not exist."
  }
}
```

**返回结果**

| 状态码 | 状态码含义                                                   | 说明       | 数据模型 |
| ------ | ------------------------------------------------------------ | ---------- | -------- |
| 200    | [OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)      | 成功       | Inline   |
| 400    | [Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1) | 请求有误   | Inline   |
| 404    | [Not Found](https://tools.ietf.org/html/rfc7231#section-6.5.4) | 记录不存在 | Inline   |

**返回数据结构**

状态码 **200**

| 名称          | 类型    | 必选 | 约束 | 中文名   | 说明 |
| ------------- | ------- | ---- | ---- | -------- | ---- |
| » data        | object  | true | none |          | none |
| »» token      | string  | true | none | token    | none |
| »» expireTime | integer | true | none | 失效时间 | none |

状态码 **4xx**

| 名称       | 类型   | 必选 | 约束 | 中文名   | 说明 |
| ---------- | ------ | ---- | ---- | -------- | ---- |
| » error    | object | true | none |          | none |
| »» code    | string | true | none | 错误码   | none |
| »» message | string | true | none | 错误信息 | none |

### 2.2 统一的错误码（ErrorCode）

提供方对OPENAPI认证不通过时，使用统一的错误码，能够帮助调用方定位认证失败的原因

| 错误代码        | 响应码 | 描述               |
| --------------- | ------ | ------------------ |
| BadArgument     | 400    | 提供的参数无效     |
| BadTimeArgument | 400    | 提供的时间参数无效 |
| BadSignArgument | 400    | 提供的签名参数无效 |
| InvalidToken    | 401    | 提供的Token无效    |
| NotFound        | 404    | 提供的appId不存在  |

### 2.3 重要规则说明

1. appSecret 不参与实际请求

2. 只有Query参数参与签名，Body参数不参与签名

   >一些Web应用服务器（如Tomcat）只允许读取一次请求体（RequestBody）。如果在鉴权时读取请求体，那么鉴权通过后实际接口将无法再次读取请求体。为了规避此类问题，只对Query参数进行签名。

3. 如果参数值为空不参与签名

4. 如果参数值不为空，但参数值前后存在空白字符，则需剔除前后的空白字符后再进行签名

5. 如果数组类型的参数，那么最终加签的参数为数组最后一个元素

6. 参数名区分大小写

7. Hash算法为SHA-256；$Hash(str)$表示对字符串str计算SHA-256摘要值后转为16进制的字符串



## 3. 示例

假设请求OPENAPI：https://localhost/openapi/v1.0/demo?productCode=&standardVersion=&arch=&os=

假设要传送的参数如下：

```
QueryParam：
	productCode=032
	standardVersion=V5.1R23C02
	arch=arm
	os=centos
Header：
	timestamp=1701759625014
	token=7e09f5d9-84e6-4054-9788-c275e230f51f
	nonce=5wf4v8
```

第一步：对Query参数名按照ASCII字典序排序，再按照key=value的格式对所有参数用"&"符号进行拼接，得到 paramPairStr：

paramPairStr="arch=arm&os=centos&productCode=032&standardVersion=V5.1R23C02"

第二步：按照 paramPairStr+"&"+timestamp+token+nonce的方式拼接得到中间结果 signStr：

signStr="arch=arm&os=centos&productCode=032&standardVersion=V5.1R23C02​&**1701759625014**7e09f5d9-84e6-4054-9788-c275e230f51f**5wf4v8**"

第三步：使用约定好的Hash算法计算signStr的摘要值作为 sign：

sign=Hash(signStr)="56c8ae4140604b85a703c904bcdbe63f51660e5bfb6794c878a2afb3249e181e"

最终得到最终发送的参数如下：

```
QueryParam：
	productCode=032
	standardVersion=V5.1R23C02
	arch=arm
	os=centos
Header：
	timestamp=1701759625014
	token=7e09f5d9-84e6-4054-9788-c275e230f51f
	nonce=5wf4v8
  sign=56c8ae4140604b85a703c904bcdbe63f51660e5bfb6794c878a2afb3249e181e
```



## 4. 兼容

如果OPENAPI提供方存在已实现的鉴权规则，那么可以根据实际情况对此鉴权规则进行兼容，以下兼容方式仅供参考。

### 4.1 根据鉴权参数兼容

若OPENAPI提供方现有的鉴权参数不为token请求头（Header），则可根据OPENAPI是否存在token请求头来判断调用方使用的是哪种鉴权规则，从而实现两种鉴权规则的兼容。

### 4.2 根据OPENAPI的URL进行兼容

若不同鉴权规则开放的接口采用了不同的URL parttern（如URL前缀不同）区分，则可先对URL进行解析，识别出该URL对应的鉴权规则，从而实现兼容。

### 4.3 链式兼容

先尝试使用旧的鉴权规则，若鉴权不通过再使用新的鉴权规则。

