# das-rest-sdk 接入指南

## 1、简介

欢迎使用我们的REST SDK！本实践基于[《REST接口规范》](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%2525E6%25258C%252587%2525E5%25258D%252597) 旨在帮助开发者简化与我们的REST API的集成过程。通过使用本SDK，您可以更方便地进行API调用，处理请求和响应，并管理错误。

## 2、使用前提

本SDK是基于JDK8 / JDK11 / JDK17

## 3、使用

### Maven

如果您使用Maven作为构建工具，可以在您的`pom.xml`文件中添加以下依赖：

```xml
<!-- JDK8 -->
<dependency>
  <groupId>com.dbapp.rest</groupId>
  <artifactId>das-rest-sdk</artifactId>
  <version>1.0.1-jdk8</version>
</dependency>
<!-- JDK11 -->
<dependency>
    <groupId>com.dbapp.rest</groupId>
    <artifactId>das-rest-sdk</artifactId>
    <version>1.0.1-jdk11</version>
</dependency>
<!-- JDK17 -->
<dependency>
    <groupId>com.dbapp.rest</groupId>
    <artifactId>das-rest-sdk</artifactId>
    <version>1.0.1-jdk17</version>
</dependency>


<!-- 注意： 如果引入后项目启动异常(1.0.0版本)，可能是openfeign冲突,请排除依赖 -->
<dependency>
    <groupId>com.dbapp.rest</groupId>
    <artifactId>das-rest-sdk</artifactId>
    <version>...</version>
    <exclusions>
        <exclusion>
            <groupId>org.springframework.cloud</groupId>
            <artifactId>spring-cloud-starter-openfeign</artifactId>
        </exclusion>
    </exclusions>
</dependency>

```

### Gradle

如果您使用Gradle作为构建工具，可以在您的`build.gradle`文件中添加以下依赖：

```groovy
// JDK8
implementation 'com.dbapp.rest:das-rest-sdk:1.0.0-jdk8'
// JDK11
implementation 'com.dbapp.rest:das-rest-sdk:1.0.0-jdk11'
// JDK17
implementation 'com.dbapp.rest:das-rest-sdk:1.0.0-jdk17'
```



## 4、快速开始

### 4.1、基础配置或功能开关

1、启动注解：@EnableDasRest注解。开启Rest相关功能的依赖对象扫描，比如自动注入Sort、Page、全局异常、api统计等。

2、基本application配置 ：

```
das.rest.enable=true  // 默认true，开启全局拦截RestfulApiException异常:RestfulApiExceptionHandler 
das.rest.scan.enable=false  // 默认false，开启api统计
das.rest.response.advice.enable=false //默认false ，开启 ResponseBodyAdvice 快速解析响应，详见5.3
```

3、关于分页插件，见5.2.6

## 5、REST规范实践

### 5.1、基础原则

#### 5.1.1、[状态码/状态码表](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%2525E6%25258C%252587%2525E5%25258D%252597%253Fid%253D_67-%2525e7%25258a%2525b6%2525e6%252580%252581%2525e7%2525a0%252581%2525e2%252598%252586)

SDK里的状态码和错误码

http状态码可以使用com.dbapp.rest.http.HttpStatus

ErrorCode必须使用com.dbapp.rest.response.ErrorCode里定义的，这些错误码来源于[规范](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%2525E6%25258C%252587%2525E5%25258D%252597)，如果规范错误码不够使用，可以提出，我们会考虑新增。

SuccessCode必须使用com.dbapp.rest.response.SuccessCode里定义。

#### 5.1.2、[响应格式](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%2525E6%25258C%252587%2525E5%25258D%252597%253Fid%253D_68-%2525e5%252593%25258d%2525e5%2525ba%252594%2525e6%2525a0%2525bc%2525e5%2525bc%25258f%2525e2%252598%252586)

没有特殊需求，推荐使用JSON作为传输格式。JSON属性名应该采用驼峰命名规范 ([RFC3864](https://tools.ietf.org/html/rfc3864))。

默认的响应格式（没有指定Accept）应该是application/json。

##### 5.1.2.1、成功响应样例

成功响应**必须**是单个JSON对象。为保证一致性，该对象必须有一个名为“data”的键，其值类型按不同情况返回,其它字段看需求返回。

如果是单个资源的获取，“data”必须是单个JSON对象。

如果是多个资源的获取，“data”必须是包含0-n个对象的JSON数组。

有两种接口响应的构造方式，推荐使用SuccessCode方式，更快捷明了。

```java
//方式一（推荐）：
public SuccessResponse<String> getProducts(@RequestParam String productName) {
    //......
    //return SuccessResponse<String> response = SuccessCode.Success.data("id").build();
    //SuccessResponse<ProductDTO> response = SuccessCode.Accepted.data(productDTO).build();
    SuccessResponse<String> response = SuccessCode.Accepted.resource(resource).build();
}

//方式二：
public SuccessResponse<List<ProductDTO> getProducts(@RequestParam String productName) {
    //......
    return SuccessResponse<List<ProductDTO>> response = SuccessResponse.success(result).total(100L).build();
}
```

```json
{
  "data": {
    "agentType":"soc",
    "softVersion":"3.0.1",
    "config":"",
    "version":1611054006000
  },
  "$page":5,
  "$size":10,
  "total":100,
  "$orderBy":"name desc,createTime"
}
```

或者

```json
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

##### 5.1.2.2、错误响应规范

有两种接口响应的构造方式，推荐使用ErrorCode方式，更快捷明了。

```java
//方式一（推荐）：
public SuccessResponse<List<ProductDTO> getProducts(@RequestParam String productName) {
    //......
    return ErrorResponse response = ErrorCode.InternalError.build();
}

//方式二：
public SuccessResponse<List<ProductDTO> getProducts(@RequestParam String productName) {
    //......
    ErrorResponse response = ErrorResponse.error(ErrorCode.BadArgument).message("xxx 参数异常").build();
}
```

```json
{
    "error": {
        "code":"GatewayTimeout",
        "message":"GatewayTimeout "
    }
}
// 或者
{
  "error": {
    "code": "BadArgument",
    "message": "Previous passwords may not be reused",
    "target": "password",
    "innerError": {
      "code": "PasswordError",
      "innererror": {
        "code": "PasswordDoesNotMeetPolicy"
      }
    }
  }
}
// 或者
{
	"error": {
		"code": "BadArgument",
		"details": [
			{
				"code": "BadUserArgument",
				"message": "user name error",
				"target": "http://..."
			}
		],
		"innerError": {
			"code": "BadUserArgument",
			"details": {
				"maxUserLength": 50,
				"minUserLength": 2
			},
			"innerError": {
				"code": "UserNameLengthError",
				"details": {
					"maxUserLength": 5000,
					"minUserLength": 200
				}
			}
		},
		"message": "errorData.name is null"
	},
	"success": false
}


```




### 5.2、[资源集合使用方法](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_7-资源集合)

资源集合根据业务场景及具体数据定义，但部分子资源对多数资源来说通用的，此处定义的通用子资源如涉及，每个资源定义必须遵守。

[files（文件子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_741-files（文件子资源）)、[synchronizations（同步子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_742-synchronizations（同步子资源）)、[connectivities（连通性子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_743-connectivities（连通性子资源）)、[restorations（重置子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_744-restorations（重置子资源）)、[ options（选项子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_745-options（选项子资源）)

#### 5.2.1、**获取特定资源的文件子资源（导出特定资源）**

```java
@GetMapping("/{fileId}/files")
public SuccessResponse<String> files(@PathVariable String fileId, @RequestParam String type) {
    //......
    return SuccessResponse.success(fileId).build();
}
```

#### 5.2.2、**获取特定资源的文件子资源-[同步子资源](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%2525E6%25258C%252587%2525E5%25258D%252597%253Fid%253D_742-synchronizations%2525ef%2525bc%252588%2525e5%252590%25258c%2525e6%2525ad%2525a5%2525e5%2525ad%252590%2525e8%2525b5%252584%2525e6%2525ba%252590%2525ef%2525bc%252589)**

```
@PostMapping("/synchronizations")
public SuccessResponse<String> getSynchronizations(@PathVariable String from) {
    Resource<Task> resource = new SynchronizationResource(new Task("/api/v1.0/assets/synchronizations/12345", "12345"));
   	return SuccessCode.Accepted.resource(resource).build();
}
```

```
{
  "task": {
    "href": "/api/v1.0/assets/synchronizations/12345",
    "id": "12345"
  },
  "data":"success"
}
```

#### 5.2.3、获取特定资源的文件子资源-[选项子资源](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%2525E6%25258C%252587%2525E5%25258D%252597%253Fid%253D_745-options%2525ef%2525bc%252588%2525e9%252580%252589%2525e9%2525a1%2525b9%2525e5%2525ad%252590%2525e8%2525b5%252584%2525e6%2525ba%252590%2525ef%2525bc%252589)

```
@GetMapping("/options")
public SuccessResponse<List<ProjectData>> getOptions(@RequestBody List<Option> options) {
    //查询数据
    return SuccessCode.Success.data(new ArrayList()).build();
}
```

```json
Accept: application/json
[
	{
        "label":"projectEName",
        "value":"projectName"
    },
    {
        "label":"key",
        "value":"keyName"
    }
]
```

```
Accept: application/json
{	
 	"data": [
    		{
                "errorName": null,
                "projectName": "张三数据库",
                "projectEName": "zsPG",
                "key": "ssdfsdasadw",
                "keyName": "我是name",
                "age": null
    		}
    	],
    "mapping": [
        {
            "label": "projectEName",
         	"value": "projectName"
        },
        {
            "label": "key",
            "value": "keyName"
        }
    ],
    "$page": 1,
    "$size": 10,
   	"total": 100,
    "$orderBy": "projectName"
}
```

#### 5.2.4、[排序使用](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_75-排序)

排序的参数需要放在请求的url里，并以"$orderBy"作为kye，在请求方法里，加入 Sort 参数即可自动导入属性值。

配置启动：@EnableDasRest

```
GET https://das.com/api/v1.0/agents?$orderBy=name
GET https://das.com/api/v1.0/agents?$orderBy=name desc
GET https://das.com/api/v1.0/agents?$orderBy=name desc,createTime
```

```
@GetMapping("")
public SuccessResponse<List<ProductDTO>> getProductDTO(Sort sort) {
    // ......
	return SuccessResponse.success(result).build();
}
```

```java
GET https://das.com/api/v1.0/agents?$orderBy=name desc,createTime
Accept: application/json
HTTP/1.1 200 OK
Content-Type: application/json
{
  "$orderBy": "name desc,createTime",
  "data": [...]
}
```

#### 5.2.5、[分页](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_76-分页)

客户端可以使用 *$page*和*$size*查询参数来指定页数和每页的数量，在请求方法里，加入 Page 参数即可自动导入属性值。

配置启动：@EnableDasRest

```java
@GetMapping("")
public SuccessResponse<List<ProductDTO>> getProductDTO(Page page) {
	// ......
    return SuccessResponse.success(result).total(100L).build();
}
```

```
GET https://das.com/api/v1.0/agents?$page=5&$size=10 

HTTP/1.1 Accept: application/json 
HTTP/1.1 200 OK 
Content-Type: application/json 

{
	"$page":5,
	"$size":10,
	"total":100,
	"data":[
		...
	]
}
```

#### 5.2.6、 [分页插件](https://baomidou.com/plugins/pagination/)
mybatis-plus 分页插件集成示例
```java

@Configuration
//@MapperScan("scan.your.mapper.package")
public class MybatisPlusConfig {

    /**
     * 添加分页插件
     */
    @Bean
    public MybatisPlusInterceptor mybatisPlusInterceptor() {
        MybatisPlusInterceptor interceptor = new MybatisPlusInterceptor();
        interceptor.addInnerInterceptor(new PaginationInnerInterceptor(DbType.MYSQL)); // 如果配置多个插件, 切记分页最后添加
        // 如果有多数据源可以不配具体类型, 否则都建议配上具体的 DbType
        return interceptor;
    }
}

```




### 5.3、[JSON标准化](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_8-json标准化)

空值约束，避免null导致的NPE

```
{
	"strKey":"",
	"intKey": 0,
	"boolKey": false,
	"Objectkey": {},
	"arrayKey": []
}
```

持续时长

```java
// 解析“P3Y6M4DT12H30M5S”，生成日期和时间对象，3年6个月4天和12小时30分钟5秒 
Pair<Period, Duration> pair = TimeUtil.periodDuration("P3Y6M4DT12H30M5S");
Assert.assertEquals(Period.of(3, 6, 4), pair.getLeft());
Assert.assertEquals(Duration.ofHours(12).plusMinutes(30).plusSeconds(5), pair.getRight());

// 解析“P3Y6M4DT”，生成日期对象，3年6个月4天
Pair<Period, Duration> pair = TimeUtil.periodDuration("P3Y6M4DT");
Assert.assertEquals(Period.of(3, 6, 4), pair.getLeft());

// 解析“P3Y4D”，生成日期对象，3年4天 
Pair<Period, Duration> pair2 = TimeUtil.periodDuration("P3Y4D");
Assert.assertEquals(Period.of(3, 0, 4), pair2.getLeft());

// 解析“PT12H30M5S”，生成时间对象，12小时30分钟5秒 
Pair<Period, Duration> pair = TimeUtil.periodDuration("PT12H30M5S");
Assert.assertEquals(Duration.ofHours(12).plusMinutes(30).plusSeconds(5), pair.getRight());

// 解析“PT12H30M”，生成时间对象，12小时30 
Pair<Period, Duration> pair2 = TimeUtil.periodDuration("PT12H30M");
Assert.assertEquals(Duration.ofHours(12).plusMinutes(30), pair2.getRight());
```



### 5.4、[常用字段名称](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_109-常用字段名称)

com.dbapp.rest.field.CommonField![image-20240702152839324](assets/CommonField.png)

com.dbapp.rest.field.CommonTermField  （来源于“DAS-KB 安全知识库管理平台-国际化管理-术语管理-网络安全专业术语”）

![image-20240702152804407](assets/CommonTermField.png)


## 6、[OpenApi鉴权实践](https://scc.das-security.cn/#/docs/specifications?position=%2525E5%2525BC%252580%2525E6%252594%2525BE%2525E6%25258E%2525A5%2525E5%25258F%2525A3%2525E7%2525BB%25259F%2525E4%2525B8%252580%2525E9%252589%2525B4%2525E6%25259D%252583%2525E8%2525A7%252584%2525E8%25258C%252583)

1、具体规范流程参考：**安全能力研发部**-[研发中心规范与公约](https://scc.das-security.cn/docs/das-rds/#/?id=研发中心规范与公约)-[《开放接口统一鉴权规范》](https://scc.das-security.cn/docs/das-rds/#/开放接口统一鉴权规范)

2、服务端需要实现com.dbapp.rest.openapi.IOpenApiToken和com.dbapp.rest.openapi.IOpenApiCheck 接口，里面提供了基础实现示例，可以直接使用或者自己重新实现。客户端按服务端要求实现token获取与传参，以下是简单示例。

### 2.1、 服务端代码示例：

```java
@RestController
@RequestMapping("/openapi/v1.0")
public class ServerController implements IOpenApiToken {

    @Override
    @GetMapping("/app-tokens")
    public ApiResponse<AppToken> appTokens(@RequestParam String appId,
                                           @RequestHeader("timestamp") String timestamp,
                                           @RequestHeader("sign") String sign) {
        return defaultAppTokens(appId, timestamp, sign);
    }

    @Override
    private AppToken getAndSaveToken(String appId) {
        String token = DasApiUtil.generateRandomString(16);
        long expireTime = System.currentTimeMillis() + 300_000L;
        MyCache.APP_TOKEN_MAP.put(token, expireTime);
        MyCache.APP_INFO_MAP.put(token, new AppInfo(appId, OpenApiConstant.APP_SECRET));
        MyCache.TOKEN_NONCE_MAP.put(token, new HashSet<>());
        return new AppToken(token, expireTime);
    }

    @Override
    private AppInfo getAppInfo(String appId) {
        if (OpenApiConstant.APPID.equals(appId)) {
            return new AppInfo(OpenApiConstant.APPID, OpenApiConstant.APP_SECRET);
        }
        return null;
    }

}
```

```java
@Component
public class MyInterceptor implements HandlerInterceptor, IOpenApiCheck {

    private final static Logger logger = LoggerFactory.getLogger(MyInterceptor.class);
    private static Map<String, String> map = new HashMap<>();

    @Override
    public boolean preHandle(HttpServletRequest request, HttpServletResponse response, Object handler)
            throws Exception {
        if ("/openapi/v1.0/app-tokens".equals(request.getRequestURI())) {
            //放开获取token
            logger.warn("重新获取 token");
            return true;
        } else if (request.getRequestURI().startsWith("/openapi")) {
			
            //鉴权
             String parameterConcat = DasApiUtil.concatSignString(request.getQueryString());
            checkOpenApi(sign, parameterConcat, timestamp, token, nonce);

        } else {
            //TODO
        }
        return true;
    }

    private Map<String, Long> tokenMap() {
        return MyCache.APP_TOKEN_MAP;
    }

    @Override
    public void checkSign(String sign, String parameterConcat, String timestamp, String token, String nonce) {
        if (!StringUtils.hasLength(sign)) {
            throw new RuntimeException("缺少sign");
        }
        //判断 加签是否正确
        String signString = parameterConcat + timestamp + token + nonce;
        String signature = DasApiUtil.getSHA256Hash(signString);
        if (sign.equals(signature)) {
            logger.warn("鉴权成功 token");
        } else {
            throw new RuntimeException("签名验证失败");
        }
    }


    @Override
    public void checkToken(String token) {
        if (!StringUtils.hasLength(token) || !tokenMap().containsKey(token)) {
            throw new RuntimeException("token 不存在");
        }
        long currentTime = System.currentTimeMillis();
        if (currentTime > tokenMap().get(token)) {
//            MyCache.APP_TOKEN_MAP.remove(token);
//            MyCache.APP_INFO_MAP.remove(token);
//            MyCache.TOKEN_NONCE_MAP.remove(token);
            throw new RuntimeException("token 已过期");
        }
    }

    @Override
    public void checkNonce(String nonce) {
        if (!StringUtils.hasLength(nonce)) {
            throw new RuntimeException("缺少nonce");
        }
        //判断接口是否重放
        Set<String> nonces = MyCache.TOKEN_NONCE_MAP.get(nonce);
        if (nonces.contains(nonce)) {
            throw new RuntimeException("重复 nonce");
        } else {
            nonces.add(nonce);
        }
    }

    @Override
    public void checkTimestamp(String timestamp) {
        if (!StringUtils.hasLength(timestamp)) {
            throw new RuntimeException("缺少timestamp");
        }
        if (13 != timestamp.length()) {
            throw new RuntimeException("timestamp 格式不正确，请使用13位毫秒时间戳！");
        }
    }

}
```

### 2.2、 客户端代码示例：

```java
public class OpenApiHttpUtil {

    private static String hostname = "localhost:8002";
    private static String APP_ID = "meIsAppId";
    private static String APP_SECRET = "meIsAppSecret";
    private final static Logger logger = LoggerFactory.getLogger(OpenApiHttpUtil.class);

    private final static AppToken appToken = new AppToken();

    //获取 token
    public static AppToken appTokens(Long timestamp) {
        //判断token是否在有效期内
        if (Objects.nonNull(appToken.getExpireTime()) && appToken.getExpireTime() - 5000 > timestamp) {
            logger.debug("使用原有的 token");
            return appToken;
        }
        logger.debug("重新获取 token");
        // 生成token需要的sign
        String sign = DasApiUtil.getSHA256Hash(timestamp + APP_ID + APP_SECRET);
        HttpResponse res = HttpRequest.get(hostname + "/openapi/v1.0/app-tokens?" + OpenApiConstant.APPID + "=" + APP_ID)
                .header(OpenApiConstant.TIMESTAMP, String.valueOf(timestamp))
                .header(OpenApiConstant.SIGN, sign)
                .execute();
        JSONObject jsonObject = JSONObject.parseObject(res.body());
        logger.debug("获取的 token = {}", jsonObject);
        if (jsonObject.containsKey("error")) {
            logger.error(jsonObject.toJSONString());
            throw new RuntimeException("获取token失败");

        }
        AppToken data = JSONObject.parseObject(jsonObject.getString("data"), AppToken.class);
        appToken.copy(data);
        return appToken;
    }

	//正常使用 token
    public static String doGet(String url, Map<String, String> param) {
        if (!StringUtils.hasLength(url)) {
            return "";
        }
        long timestamp = System.currentTimeMillis();
        AppToken appToken = appTokens(timestamp);
        String parameterConcat = DasApiUtil.concatSignString(param);
        String nonce = DasApiUtil.generateRandomString(6);
        String signString = parameterConcat + timestamp + appToken.getToken() + nonce;
        // 生成 openApi 需要的sign
        String signature = DasApiUtil.getSHA256Hash(signString);

        HttpResponse res = HttpRequest.get(hostname + url +
                        (url.contains("?") ? "$" : "?")
                        + mapToString(param))
                .header(OpenApiConstant.TIMESTAMP, String.valueOf(timestamp))
                .header(OpenApiConstant.TOKEN, appToken.getToken())
                .header(OpenApiConstant.NONCE, nonce)
                .header(OpenApiConstant.SIGN, signature)
                .execute();
        JSONObject jsonObject = JSONObject.parseObject(res.body());
        logger.info("{} 的结果 {}", url, jsonObject.toJSONString());
        return jsonObject.getString("data");
    }

    //参数格式转换
    public static String mapToString(Map<String, String> map) {
        if (CollectionUtils.isEmpty(map)) {
            return "";
        }
        return map.entrySet().stream()
                .filter(entry -> StringUtils.hasLength(entry.getKey()) && StringUtils.hasLength(entry.getValue()))
                .map(entry -> entry.getKey() + "=" + entry.getValue())
                .collect(Collectors.joining("&"));
    }

}
```

