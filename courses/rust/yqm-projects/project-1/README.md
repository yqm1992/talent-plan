# PNA Rust Project 1: The Rust toolbox

实现了基于内存的KV store API，以及命令行解析工具，支持set，get，rm命令

### 编译流程
- 编译：cargo build
- 可执行文件路径：target/debug/kvs

### 运行示例：
```bsh 运行示例
# get命令
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-1$ ./target/debug/kvs get 1
unimplemented

# set命令
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-1$ ./target/debug/kvs set 1 1
unimplemented

# rm命令
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-1$ ./target/debug/kvs rm 1
unimplemented

# 错误的set命令
yangqiuming@yangqiumingsmbp:~/work/talent-plan/courses/rust/yqm-projects/project-1$ ./target/debug/kvs set 1
error: The following required arguments were not provided:
    <value>

USAGE:
    kvs set <key> <value>

For more information try --help
```