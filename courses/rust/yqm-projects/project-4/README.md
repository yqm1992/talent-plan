# PNA Rust Project 4: Concurrency and parallelism

基于简单自定义C/S协议，创建一个多线程的,持久化的kv store server，以及对应的client
- 实现一个能够在多线程间shared的kv-engine
- 实现简单的线程池，并在server端添加对多线程的支持

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

通过kvs-client访问kv server

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