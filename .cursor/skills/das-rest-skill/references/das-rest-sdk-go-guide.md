## 1、简介

欢迎使用我们的REST SDK！本实践基于[《REST接口规范》](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%25E6%258C%2587%25E5%258D%2597) 旨在帮助开发者简化与我们的REST API的集成过程。通过使用本SDK，您可以更方便地进行API调用，处理请求和响应，并管理错误。

## 2、使用前提

本SDK基于GO 1.20

## 3、使用

使用公司go module源：

https://ci.das-security.cn/repository/ah_go/

```shell
go env -w GOPROXY=https://ci.das-security.cn/repository/ah_go/,direct GOSUMDB=off
```

添加das-rest-sdk-go依赖

```shell
go get gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go@v1.0.1
```

## 4、快速开始

将go自带的http.Handler使用rest.Handler进行包装，注册到http.HandleFunc路由中

```go
package main

import (
	"fmt"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/handler"
	"net/http"
)

type Hello struct {
	Message string `json:"message"`
}

// HelloHandler is a simple HTTP handler function that writes a response
func HelloHandler(r *http.Request) (*Hello, error) {
	return &Hello{
		Message: "Hello, World!",
	}, nil
}

func main() {
	// Register HTTP handlers for different routes
	http.HandleFunc("/hello", handler.Handler(HelloHandler))
	// Start the HTTP server on port 8080
	fmt.Println("Starting server on :8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		fmt.Printf("Error starting server: %v\n", err)
	}
}

```

## 5、REST规范实践

### 5.1、基础原则

#### [5.1.1、](https://scc.das-security.cn/docs/das-rds/#/das-rest-sdk接入指南?id=_511、状态码状态码表)[状态码/状态码表](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%25E6%258C%2587%25E5%258D%2597%3Fid%3D_67-%25e7%258a%25b6%25e6%2580%2581%25e7%25a0%2581%25e2%2598%2586)

SDK里的状态码和错误码

http状态码可以使用net/http.StatusXXX

ErrorCode必须使用das-rest-sdk-go/rest.ErrorCodes里定义的，这些错误码来源于[规范](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%25E6%258C%2587%25E5%258D%2597)，如果规范错误码不够使用，可以提出，我们会考虑新增。

SuccessCode必须使用das-rest-sdk-go/rest.SuccessCodes里定义。

#### [5.1.2、](https://scc.das-security.cn/docs/das-rds/#/das-rest-sdk接入指南?id=_512、响应格式)[响应格式](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%25E6%258C%2587%25E5%258D%2597%3Fid%3D_68-%25e5%2593%258d%25e5%25ba%2594%25e6%25a0%25bc%25e5%25bc%258f%25e2%2598%2586)

没有特殊需求，推荐使用JSON作为传输格式。JSON属性名应该采用驼峰命名规范 ([RFC3864](https://tools.ietf.org/html/rfc3864))。

默认的响应格式（没有指定Accept）应该是application/json。

##### 5.1.2.1、成功响应样例

成功响应**必须**是单个JSON对象。为保证一致性，该对象必须有一个名为“data”的键，其值类型按不同情况返回,其它字段看需求返回。

如果是单个资源的获取，“data”必须是单个JSON对象。

如果是多个资源的获取，“data”必须是包含0-n个对象的JSON数组。

```go
// 方式一：直接返回Data对象，rest.Handler会将结果包装为restful格式json，httpStatus默认200
/*
   响应json：
   HTTP/1.1 200 OK
   {
        "data": {
            "message": "Hello, World!"
        }
    }
*/
func HelloHandler(r *http.Request) (*Hello, error) {
	return &Hello{
		Message: "Hello, World!",
	}, nil
}

// 方式二：返回*rest.Response[T]包装对象，结果与方式一相同，可自定义SuccessCode
func HelloHandler(r *http.Request) (*rest.Response[*Hello], error) {
    hello := &Hello{
		Message: "Hello, World!",
	}, nil
    return rest.Success(hello).Build(), nil
    
    // 可通过builder方式变更SuccessCode
    // Created对应http status为201
    return rest.Created(hello).Build(), nil
}

// 方式三：通过结构体字面量直接创建rest.Response
func HelloHandler(r *http.Request) (*rest.Response[*Hello], error) {
    return &rest.Response[*Hello]{
                Success: rest.CreatedCode,
                Data: &Hello{
                    Message: "Hello, World!",
                },
            }, nil
}
```

##### 5.1.2.2、错误响应规范

通过error方式返回错误响应。

```go
// 方式一：通过builder方式构建
/*
   响应json：
   HTTP/1.1 400 Bad Request
   {
        "error": {
            "code": "BadArgument",
            "message": "参数错误"
        }
    }
*/
func HelloHandler(r *http.Request) (*Hello, error) {
	return nil, rest.BadArgument.Message("参数错误").Build()
}

// 方式二：通过结构体字面量构建error
func HelloHandler(r *http.Request) (*Hello, error) {
	return nil, &rest.Error{
			Code:    rest.BadArgument,
			Message: "参数错误",
		}
}

// 方式三：返回任意error，rest.Handler会将结果包装为restful格式json，httpStatus默认500
/*
   响应json：
   HTTP/1.1 500 Internal Server Error
   {
        "error": {
            "code": "InternalError",
            "message": "服务端错误"
        }
    }
*/
func HelloHandler(r *http.Request) (*Hello, error) {
	return nil, errors.New("服务端错误")
}
```

### [5.2、](https://scc.das-security.cn/docs/das-rds/#/das-rest-sdk接入指南?id=_52、资源集合使用方法)[资源集合使用方法](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_7-资源集合)

资源集合根据业务场景及具体数据定义，但部分子资源对多数资源来说通用的，此处定义的通用子资源如涉及，每个资源定义必须遵守。

[files（文件子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_741-files（文件子资源）)、[synchronizations（同步子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_742-synchronizations（同步子资源）)、[connectivities（连通性子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_743-connectivities（连通性子资源）)、[restorations（重置子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_744-restorations（重置子资源）)、[ options（选项子资源）](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_745-options（选项子资源）)

#### 5.2.1、**获取特定资源的文件子资源（导出特定资源）**

```go
// DownloadHandler 处理下载请求
func DownloadHandler(w http.ResponseWriter, r *http.Request) {
	// 获取文件
	file, err := getFile(r)
	if err != nil {
		err := rest.NotFound.Message("file not found").Build()
        // restful handler处理error输出
		handler.HandleResponse(context.Background(), nil, err, w)
		return
	}

	// 设置响应头
	w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=%s", file.FileName))
	w.Header().Set("Content-Type", "application/octet-stream")

	// 将文件内容写入响应体
	http.ServeContent(w, r, file.FileName, file.ModTime(), file)
}
```

#### 5.2.2、获取特定资源的文件子资源-[同步子资源](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%25E6%258C%2587%25E5%258D%2597%3Fid%3D_742-synchronizations%25ef%25bc%2588%25e5%2590%258c%25e6%25ad%25a5%25e5%25ad%2590%25e8%25b5%2584%25e6%25ba%2590%25ef%25bc%2589)

```go
func SyncHandler(r *http.Request) (*resource.Sync, error) {
	return &resource.Sync{
		Task: &resource.Task{
			Id:   "1",
			Href: "/api/v1.0/assets/synchronizations/12345",
		},
	}, nil
}
```

```http
HTTP/1.1 202 Accepted

{
	"task": {
		"id": "1",
		"href": "/api/v1.0/assets/synchronizations/12345"
	}
}
```

#### 5.2.3、获取特定资源的文件子资源-[选项子资源](https://scc.das-security.cn/#/docs/specifications?position=DAS-REST-API%25E6%258C%2587%25E5%258D%2597%3Fid%3D_745-options%25ef%25bc%2588%25e9%2580%2589%25e9%25a1%25b9%25e5%25ad%2590%25e8%25b5%2584%25e6%25ba%2590%25ef%25bc%2589)

显式:

```go
func OptionHandler(r *http.Request) (*resource.Options, error) {
	return &resource.Options{
		{
			Value: "key1",
			Label: "label1",
		},
		{
			Value: "key2",
			Label: "label2",
		},
	}, nil
}
```

```http
HTTP/1.1 200 OK

[
	{
		"value": "key1",
		"label": "label1"
	},
	{
		"value": "key2",
		"label": "label2"
	}
]
```

映射式:

```go
func OptionMappingHandler(r *http.Request) (*resource.OptionMapping[[]*Hello], error) {
	return &resource.OptionMapping[[]*Hello]{
		Data: []*Hello{
			{
				Key: "test",
				Name: "测试",
			},
		},
		Mapping: &resource.Option{
			Value: "key",
			Label: "name",
		},
	}, nil
}
```

```http
HTTP/1.1 200 OK

{
	"data": [
		{
			"key": "test",
			"name": "测试"
		}
	],
	"mapping": {
		"value": "key",
		"label": "name"
	}
}
```

#### 5.2.4、[分页/排序使用](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_75-排序)

客户端可以使用 "$page"和"$size"查询参数来指定页数和每页的数量，在请求方法里，加入Sort 参数即可自动导入属性值。

排序的参数需要放在请求的url里，并以"$orderBy"作为key，在请求方法里，加入Sort 参数即可自动导入属性值。

使用rest.PageHandler进行包装，注册到http.HandleFunc路由中

```go
package main

import (
	"fmt"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/handler"
	"net/http"
)

// ListHandler is a simple HTTP handler function that writes a response
func ListHandler(r *http.Request, page *rest.Page, sort *rest.Sort) (*rest.Response[[]*Hello], error) {
	list := []*Hello{
		{
			Message: "Hello, World!",
		},
	}
  return rest.Success(list).Page(page).Sort(sort).Total(10).Build(), nil
}

func main() {
	// Register HTTP handlers for different routes
	http.HandleFunc("/list", handler.PageHandler(ListHandler))
	// Start the HTTP server on port 8080
	fmt.Println("Starting server on :8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		fmt.Printf("Error starting server: %v\n", err)
	}
}
```

list?\$page=1&\$size=10&\$orderBy=name desc

```http
HTTP/1.1 200 OK

{
	"data": [
		{
			"message": "Hello, World!"
		}
	],
	"total": 10,
	"$orderBy": "name desc",
	"$page": 1,
	"$size": 10
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

sdk间接依赖了三方库github.com/rickb777/date

```go
func TestTime(t *testing.T) {

	// 解析“P3Y6M4DT12H30M5S”，生成日期和时间对象，3年6个月4天和12小时30分钟5秒
	result, _ := period.Parse("P3Y6M4DT12H30M5S")
	expected := period.New(3, 6, 4, 12, 30, 5)
	if !reflect.DeepEqual(result, expected) {
		t.Errorf("expected %v, got %v", expected, result)
	}

	// 解析“P3Y6M4DT”，生成日期对象，3年6个月4天
	result, _ = period.Parse("P3Y6M4DT")
	expected = period.New(3, 6, 4, 0, 0, 0)
	if !reflect.DeepEqual(result, expected) {
		t.Errorf("expected %v, got %v", expected, result)
	}

	// 解析“P3Y4D”，生成日期对象，3年4天
	result, _ = period.Parse("P3Y4D")
	expected = period.New(3, 0, 4, 0, 0, 0)
	if !reflect.DeepEqual(result, expected) {
		t.Errorf("expected %v, got %v", expected, result)
	}

	// 解析“PT12H30M5S”，生成时间对象，12小时30分钟5秒
	result, _ = period.Parse("PT12H30M5S")
	expected = period.New(0, 0, 0, 12, 30, 5)
	if !reflect.DeepEqual(result, expected) {
		t.Errorf("expected %v, got %v", expected, result)
	}

	// 解析“PT12H30M”，生成时间对象，12小时30
	result, _ = period.Parse("PT12H30M")
	expected = period.New(0, 0, 0, 12, 30, 0)
	if !reflect.DeepEqual(result, expected) {
		t.Errorf("expected %v, got %v", expected, result)
	}
}
```



### 5.4、[常用字段名称](https://scc.das-security.cn/docs/das-rds/#/DAS-REST-API指南?id=_109-常用字段名称)

das-rest-sdk-go/field.CommonField

![image-20240912174750624](assets\image-20240912174750624.png)

das-rest-sdk-go/field.TermField（来源于“DAS-KB 安全知识库管理平台-国际化管理-术语管理-网络安全专业术语”）

![image-20240912174934087](assets\image-20240912174934087.png)

## 6、[OpenApi鉴权实践](https://scc.das-security.cn/#/docs/specifications?position=%25E5%25BC%2580%25E6%2594%25BE%25E6%258E%25A5%25E5%258F%25A3%25E7%25BB%259F%25E4%25B8%2580%25E9%2589%25B4%25E6%259D%2583%25E8%25A7%2584%25E8%258C%2583)

1、具体规范流程参考：**安全能力研发部**-[研发中心规范与公约](https://scc.das-security.cn/docs/das-rds/#/?id=研发中心规范与公约)-[《开放接口统一鉴权规范》](https://scc.das-security.cn/docs/das-rds/#/开放接口统一鉴权规范)

### 6.1、服务端代码示例

1、服务端需要实现das-rest-sdk-go/auth.AppManager管理AppInfo

2、das-rest-sdk-go/auth.DefaultTokenValidator里提供了token认证默认实现，可以直接使用或重新实现

```go
package main

import (
	"fmt"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/auth"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/handler"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/rest"
	"net/http"
)

var testAppId = "test-app"
var testSecret = "secret-123456"
var userName = "admin"

// tokenValidator is a default token validator
var tokenValidator = auth.DefaultTokenValidator(&AppManager{})

// AppManager AppInfo管理接口实现
type AppManager struct {
}

// GetApp 通过appId获取AppInfo
func (s *AppManager) GetApp(appId string) (*auth.AppInfo, error) {
	if appId != testAppId {
		return nil, rest.BadArgument.Message("appid not match").Build()
	}
	return &auth.AppInfo{
		AppId:     testAppId,
		AppSecret: testSecret,
		UserName:  userName,
	}, nil
}

type Hello struct {
	Message string `json:"message"`
}

// HelloHandler is a simple HTTP handler function that writes a response
func HelloHandler(r *http.Request) (*Hello, error) {
	return &Hello{
		Message: "Hello, World!",
	}, nil
}

func main() {
	// Create a new ServeMux
	webMux := http.NewServeMux()
	// Register HTTP handlers for different routes
	webMux.HandleFunc("/hello", handler.Handler(HelloHandler))

	mux := http.NewServeMux()
	// token验证过滤器
	authMux := tokenValidator.DefaultMiddleware(webMux)
	mux.Handle("/", authMux)

	// 设置不拦截的路由和处理函数
	// 认证获取token默认实现
	mux.HandleFunc("/token", handler.Handler(tokenValidator.GenerateToken))
	// Start the HTTP server on port 8080
	fmt.Println("Starting server on :8080")
	if err := http.ListenAndServe(":8080", mux); err != nil {
		fmt.Printf("Error starting server: %v\n", err)
	}
}
```

#### 6.1.1、自定义TokenManager

NewInMemoryTokenManager是接口TokenManager的默认实现，在内存中管理token信息，如果需要持久化可自定义实现TokenManager接口

```go
// TokenManager 是一个Token管理接口，包含生成和获取Token的方法
type TokenManager interface {
	// GenerateToken 生成Token，并保存到Token管理实例中，token认证时调用
	GenerateToken(appId, appSecret, userName string) (tokenInfo *TokenInfo, err error)
	// GetToken 根据Token获取Token信息，openapi接口请求过程中校验token有效性使用
	GetToken(token string) (tokenInfo *TokenInfo, err error)
}
```

#### 6.1.2、自定义TokenParamConverter

TokenParamConverter是预定义func类型，作用是将待校验实例转换为标准的TokenParam实例

DefaultTokenParamConverter默认实现是将http.Request转换为TokenParam，可自定义参考实现

```go
type TokenParamConverter[T any] func(request T) *TokenParam
```

### 6.2、客户端实例代码

1、das-rest-sdk-go/auth.DefaultTokenTransport里提供了token获取与拼接默认实现，可以直接使用或重新实现

```go
package main

import (
	"encoding/json"
	"fmt"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/auth"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/rest"
	"io"
	"net/http"
	"time"
)

type Hello struct {
	Message string `json:"message"`
}

// 默认token转换器
// 提供默认token认证实现
// 认证成功后缓存token，token失效自动重新获取
// 提供接口请求拼接token header逻辑
var transport = auth.DefaultTokenTransport(
	&auth.AppInfo{
		AppId:     "test-app",
		AppSecret: "secret-123456",
	},
	&auth.Config{
		ServerAddress: "http://localhost:8080",
		LoginUrl:      "/token",
	},
    // 是否忽略ssl证书
	false)

func Get[T any]() (T, error) {
	client := &http.Client{
		Transport: transport,
		Timeout:   10 * time.Minute,
	}
	// 发送 GET 请求
	getURL := "http://localhost:8080/hello"
	resp, err := client.Get(getURL)
	if err != nil {
		fmt.Printf("Error making GET request: %v\n", err)
		return *new(T), err
	}
	defer resp.Body.Close()
	// 读取 GET 响应体
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		fmt.Printf("Error reading GET response body: %v\n", err)
		return *new(T), err
	}
	fmt.Printf("GET Response:\n%s\n", body)

	var res = &rest.Response[T]{}
	err = json.Unmarshal(body, res)
	if err != nil {
		return *new(T), err
	}
	if res.Error != nil {
		return *new(T), res.Error
	}
	return res.Data, nil

}

func main() {
	hello, err := Get[Hello]()
	if err != nil {
		fmt.Printf("Error getting hello: %v\n", err)
	} else {
		fmt.Println(hello.Message)
	}
}
```

## 7、三方web框架集成

以goframe框架为例，以下为集成代码，如果有其他三方web框架可以参考实现：

### 7.1、goframe框架集成

**1、定义中间件Middleware**

解析Page、Sort参数，统一处理response响应

```go
package middleware

import (
	"github.com/gogf/gf/v2/net/ghttp"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/handler"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/rest"
	"online-hub/internal/service"
)

type sMiddleware struct {
}

func init() {
	service.RegisterMiddleware(New())
}

func New() *sMiddleware {
	return &sMiddleware{}
}

func (s *sMiddleware) RestResponseHandler(r *ghttp.Request) {
	//生成一个context
	sort, err := rest.ParseRequestSortParam(r.Request)
	if err != nil {
		handler.HandleResponse(r.Context(), nil, rest.BadArgument.Message(err.Error()).Build(), r.Response.BufferWriter)
		r.Exit()
		return
	}
	// 将解析后的参数存储到param中
	r.SetParam("$orderBy", sort)

	page, err := rest.ParseRequestPageParam(r.Request)
	if err != nil {
		handler.HandleResponse(r.Context(), nil, rest.BadArgument.Message(err.Error()).Build(), r.Response.BufferWriter)
		r.Exit()
		return
	}

	// 将解析后的参数存储到请求上下文中
	r.SetCtxVar(rest.CtxPage, page)
	r.SetCtxVar(rest.CtxSort, sort)

	r.Middleware.Next()

	// 如果已经有返回内容，那么该中间件什么也不做
	if r.Response.BufferLength() > 0 {
		return
	}
	// 默认restful response处理
	handler.HandleResponse(r.Context(), r.GetHandlerResponse(), r.GetError(), r.Response.BufferWriter)
	r.Exit()
}

```

**2、启动路由添加中间件**

```go
package main

import (
	_ "github.com/gogf/gf/contrib/drivers/sqlite/v2"
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/net/ghttp"
	"online-hub/internal/controller/resttest"
	_ "online-hub/internal/packed"
	"online-hub/internal/service"

	_ "online-hub/internal/logic"
)

func main() {
	s := g.Server()
	// 一个简单的分页路由示例
	s.Group("/das-upgrade", func(group *ghttp.RouterGroup) {
		group.Middleware(
            //集成自定义restful中间件
			service.Middleware().RestResponseHandler,
		)
		group.Bind(
			resttest.NewV1(),
		)
	})
	s.Run()
}

```

**3、定义ctrl接口**

```go
package v1

import (
	"github.com/gogf/gf/v2/frame/g"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/rest"
)

type HelloReq struct {
	g.Meta `path:"/api/v1/hello" method:"get"`
	Test   string `p:"test"`
}

type HelloRes struct {
	Message string `json:"message"`
}

type PageNameReq struct {
	g.Meta `path:"/api/v1/page" method:"get"`

	*rest.Page
	Sort *rest.Sort `p:"$orderBy"`
}

type PageNameData struct {
	Name string
}

type PageNameRes rest.Response[[]*PageNameData]
```

**4、ctrl接口实现**

```go
package resttest

import (
	"context"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/rest"
	"github.com/gogf/gf/v2/errors/gerror"
	"online-hub/api/resttest/v1"
)

func (c *ControllerV1) Hello(ctx context.Context, req *v1.HelloReq) (res *v1.HelloRes, err error) {
	if req.Test == "error1" {
        /*
           url:/das-upgrade/api/v1/hello?test=error1
           
           响应：
   		   HTTP/1.1 500 Internal Server Error
           {
                "error": {
                    "code": "InternalError",
                    "message": "test error1"
                }
            }
        */
		return nil, gerror.New("test error1")
	} else if req.Test == "error2" {
        /*
           url:/das-upgrade/api/v1/hello?test=error2
           
           响应：
           HTTP/1.1 400 Bad Request
           {
                "error": {
                    "code": "BadArgument",
                    "message": "internal error2"
                }
            }
        */
		return nil, rest.BadArgument.Message("internal error2").Build()
	} else {
        /*
           url:/das-upgrade/api/v1/hello
           
           响应：
           HTTP/1.1 200 OK
           {
                "data": {
                    "message": "Hello world!!!"
                }
            }
        */
		return &v1.HelloRes{
			Message: "Hello world!!!",
		}, nil
	}
}
```

```go
package resttest

import (
	"context"
	"github.com/gogf/gf/v2/frame/g"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/rest"
	"online-hub/api/resttest/v1"
)

func (c *ControllerV1) PageName(ctx context.Context, req *v1.PageNameReq) (res *v1.PageNameRes, err error) {
	g.Log().Infof(ctx, "sort:%s, num:%d, size:%d, offset:%d", req.Sort, req.Num, req.Size, req.Offset())
    /*
       url:/das-upgrade/api/v1/page?$page=2&$size=10&$orderBy=name desc

       响应：
       HTTP/1.1 200 OK
       {
            "data": [
                {
                    "Name": "test"
                }
            ],
            "total": 20,
            "$orderBy": "name desc",
            "$page": 2,
            "$size": 10
        }
    */
	return &v1.PageNameRes{
		Success: rest.SuccessCode,
		Data: []*v1.PageNameData{
			{
				Name: "test",
			},
		},
		Total: 20,
	}, nil
}
```

**5、OpenApi鉴权集成**

1）实现das-rest-sdk-go/auth.AppManager接口，管理AppInfo

2）创建das-rest-sdk-go/auth.TokenValidator校验器实例

3）添加GenerateToken认证获取token接口

4）中间件middleware添加ValidateToken路由过滤

```go
package middleware

import (
	"github.com/gogf/gf/v2/net/ghttp"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/auth"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/handler"
	"gitlab.info.dbappsecurity.com.cn/caphub/das-rest-sdk-go/rest"
	"online-hub/internal/service"
)

type sMiddleware struct {
}

func init() {
	service.RegisterMiddleware(New())
}

func New() *sMiddleware {
	return &sMiddleware{}
}

var testAppId = "test-app"
var testSecret = "secret-123456"
var userName = "admin"

// tokenValidator is a default token validator
var tokenValidator = &auth.TokenValidator[*ghttp.Request]{
	AppManager:   &AppManager{},
	TokenManager: auth.NewInMemoryTokenManager(),
	TokenParamConverter: func(r *ghttp.Request) *auth.TokenParam {
		return auth.DefaultTokenParamConverter(r.Request)
	},
}

// AppManager AppInfo管理接口实现
type AppManager struct {
}

// GetApp 通过appId获取AppInfo
func (s *AppManager) GetApp(appId string) (*auth.AppInfo, error) {
	if appId != testAppId {
		return nil, rest.BadArgument.Message("appid not match").Build()
	}
	return &auth.AppInfo{
		AppId:     testAppId,
		AppSecret: testSecret,
		UserName:  userName,
	}, nil
}

// GenerateToken 认证获取token
func (s *sMiddleware) GenerateToken(r *ghttp.Request) {
	token, err := tokenValidator.GenerateToken(r)
	handler.HandleResponse(r.Context(), token, err, r.Response.BufferWriter)
}

// ValidateToken 接口token校验
func (s *sMiddleware) ValidateToken(r *ghttp.Request) {
	validateToken, err := tokenValidator.ValidateToken(r)
	if err != nil {
		handler.HandleResponse(r.Context(), nil, err, r.Response.BufferWriter)
		r.Exit()
		return
	}
	r.SetCtxVar("userName", validateToken.UserName)
	r.Middleware.Next()
}

func (s *sMiddleware) RestResponseHandler(r *ghttp.Request) {
	//生成一个context
	sort, err := rest.ParseRequestSortParam(r.Request)
	if err != nil {
		handler.HandleResponse(r.Context(), nil, rest.BadArgument.Message(err.Error()).Build(), r.Response.BufferWriter)
		r.Exit()
		return
	}
	// 将解析后的参数存储到param中
	r.SetParam("$orderBy", sort)

	page, err := rest.ParseRequestPageParam(r.Request)
	if err != nil {
		handler.HandleResponse(r.Context(), nil, rest.BadArgument.Message(err.Error()).Build(), r.Response.BufferWriter)
		r.Exit()
		return
	}

	// 将解析后的参数存储到请求上下文中
	r.SetCtxVar(rest.CtxPage, page)
	r.SetCtxVar(rest.CtxSort, sort)

	r.Middleware.Next()

	// 如果已经有返回内容，那么该中间件什么也不做
	if r.Response.BufferLength() > 0 {
		return
	}
	// 默认restful response处理
	handler.HandleResponse(r.Context(), r.GetHandlerResponse(), r.GetError(), r.Response.BufferWriter)
	r.Exit()
}
```

路由配置认证获取token接口

路由添加接口token认证过滤Middleware

```go
package main

import (
	_ "github.com/gogf/gf/contrib/drivers/sqlite/v2"
	"github.com/gogf/gf/v2/frame/g"
	"github.com/gogf/gf/v2/net/ghttp"
	"online-hub/internal/controller/resttest"
	_ "online-hub/internal/logic"
	_ "online-hub/internal/packed"
	"online-hub/internal/service"
)

func main() {
	s := g.Server()
	// 一个简单的分页路由示例
	s.Group("/das-upgrade", func(group *ghttp.RouterGroup) {
		group.Middleware(
			service.Middleware().RestResponseHandler,
		)
        // 认证获取token接口
		group.GET("/token", service.Middleware().GenerateToken)
		group.Group("/", func(group *ghttp.RouterGroup) {
			group.Middleware(
                // 接口token认证过滤
				service.Middleware().ValidateToken,
			)
			group.Bind(
				resttest.NewV1(),
			)
		})
	})
	s.Run()
}
```

