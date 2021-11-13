# PNA Rust Project 2: Log-structured file I/O

实现一个持久化存储的kv store，能通过命令行进行访问
- 通过serde库实现数据的序列化/反序列化
- 实现bitcask日志类型的存储引擎（内存中保存着所有key的索引记录）
- 周期性的压缩日志，清除过期数据

### 编译流程
- 编译：cargo build
- 可执行文件路径：target/debug/kvs

### 运行示例：
```bsh 运行示例
# 执行get操作
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-2$ ./target/debug/kvs get 111
Key not found

# 执行set操作
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-2$ ./target/debug/kvs set 111 111

# 执行get操作
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-2$ ./target/debug/kvs get 111
111

# 执行rm操作
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-2$ ./target/debug/kvs rm 111

# 执行get操作
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-2$ ./target/debug/kvs get 111
Key not found
```