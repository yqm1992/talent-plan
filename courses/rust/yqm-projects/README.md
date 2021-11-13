# Practical Networked Applications in Rust

用Rust逐步实现一个基于bitcask存储引擎的多线程的kv server
- Project1 实现了基于内存的KV store API，以及命令行解析工具
- Project2 实现一个持久化存储的kv store（bitcask日志型存储），能通过命令行进行实际的kv操作
- Project3 基于简单自定义C/S协议，实现一个单线程的kv store server和对应的client
- Project4 实现基于线程池的多线程, 持久化的kv store server