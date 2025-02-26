# quanta

# 概述
一个基于lua + rust的分布式跨平台游戏服务器引擎框架！

[quanta](https://github.com/xiyoo0812/quanta.git)的rust版本，quanta使用C++17开发，quanta-rs与C++版本保持lua级别的API兼容！两个仓库lua代码完全兼容，可以快速开发。

# 优势
- 轻量级
- 简单、易上手
- 稳定性强
- 扩展性强
- 热更新
- 跨平台(WINDOWS/LINUX/MACOS)

# 编译
- windows: 在项目根目录执行nmake dev/pub
- linux：在项目根目录，执行make dev/pub。
- windows依赖nmake 编译，需要安装nmake，并设置环境变量
- 直接使用cargo编译也可以，需要安装rust，并设置环境变量

# 体验引擎
- 配置
在bin/config目录下，仿造quanta.conf生成配置实例，然后在bin目录执行configure.bat/configure.sh，会自动生成项目配置
- 自动生产配置依赖[lmake](https://github.com/xiyoo0812/lmake.git)，已经内置到项目
```shell
#linux
#需要加参数配置文件名
configure.sh quanta
#windows
configure.bat
#然后输入配置文件名
#>>quanta
```
- 执行
可以bin下的quanta.bat/quanta.sh, test.bat/test.sh体验
```shell
#linux
quanta.sh
#windows
quanta.bat
```

# 基础服务
- router: quanta框架采用星形结构，router提供路由服务。
- test: 测试组件，提供基本给你测试的服务
- dbsvr: 提供基础的数据库访问服务。
- proxy: 提供基础的http访问服务。
- cachesvr: 提供基础的数据缓存服务。
- discover: 提供服务发现功能，以及基于http提供启停、监控的服务。

# 数据库支持
- mongo
- redis
- mysql
- pgsql
- sqlite
- clickhouse

# KV存储支持
- redis
- lmdb
- smdb
- unqite

# 支持功能
- json协议支持
- protobuf协议支持
- SSL支持
- HTTP C/S支持
- TCP/UDP C/S支持
- websocket C/S支持
- xml/yaml/toml配置支持
- excel(xlsx/xlsm/csv)配置导出
- 常用压缩算法(lz4,minizip,zstd)支持
- 常用加密算法(BASE64,MD5,RSA,SHA系列,hmac系列)支持
- rpc调用机制支持
- 协议加密和压缩功能支持
- 文件系统支持
- 异步日志功能支持
- lua面向对象机制支持
- 性能/流量统计支持
- 游戏数据缓存机制支持
- 脚本文件加密机制支持
- 游戏逻辑/配置热更新机制支持
- 协程调用框架
- 游戏GM功能框架
- 服务发现功能框架
- 基于行为树的机器人测试框架
- 星型分布式服务器框架

# 辅助工具
- GMWeb工具
- 协议测试Web工具
- zipkin/jager调用链系统
- dingding/wechat/lark等webhook通知
