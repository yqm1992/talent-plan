# PNA Rust Project 3: Synchronous client-server networking


基于简单自定义C/S协议，实现一个单线程的kv store server和对应的client
- 在server端添加了日志打印功能
- 通过traits实现了可替换后端存储引擎的kv store

### 编译流程
- 编译：cargo build
- server运行路径为：target/debug/kvs-server
- client文件路径为：target/debug/kvs-server/kvs-client

### 运行示例：
首先启动server：./target/debug/kvs-server

```bsh server运行示例
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-3$ ./target/debug/kvs-server
Nov 11 15:36:00.049 INFO Server start, ServerInfo { addr: "127.0.0.1:4000", engine_name: "kvs", version: "0.1.0" }
```

通过kvs-client访问kv server：
```bsh client运行示例
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-3$ ./target/debug/kvs-client get 111
Key not found

yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-3$ ./target/debug/kvs-client set 111 111

yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-3$ ./target/debug/kvs-client get 111
111

yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-3$ ./target/debug/kvs-client rm 111

yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-3$ ./target/debug/kvs-client get 111
Key not found
```