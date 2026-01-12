# czdb-rs

<p align="center">中文文档 | <a href="./README_EN.md">English Documentation</a></p>


基于 Rust 实现的高性能 CZDB IP 地理位置查询库，支持 WASM。

## 特点

- IP 地理位置查询
- 支持 IPv4 和 IPv6 地址
- 简单易用的 API
- 两种搜索模式：Memory（最快）和 BTree（低内存）

## 性能

### Rust (原生)

**Release 模式基准测试 (5485 IPv4 / 2010 IPv6 查询):**

| 序号 | 名称               | 模式    | 总时间 (ms) | 平均时间 (ms) | 查询数 |
|-----|--------------------|--------|-------------|---------------|-------|
| 1   | Rust IPv4          | Memory | 1.46        | 0.0003        | 5485  |
| 2   | Rust IPv4 BTree    | BTree  | 1.65        | 0.0003        | 5485  |
| 3   | Rust IPv6          | Memory | 0.43        | 0.0002        | 2010  |
| 4   | Rust IPv6 BTree    | BTree  | 0.48        | 0.0002        | 2010  |

### Node.js (WASM)

**基准测试结果 (5485 IPv4 / 2010 IPv6 查询):**

| 序号 | 名称               | 模式    | 总时间 (ms) | 平均时间 (us) | 查询数 |
|-----|--------------------|--------|-------------|---------------|-------|
| 1   | WASM IPv4          | Memory | 8.75        | 1.60          | 5485  |
| 2   | WASM IPv4 BTree    | BTree  | 3.36        | 0.60          | 5485  |
| 3   | WASM IPv6          | Memory | 1.51        | 0.80          | 2010  |
| 4   | WASM IPv6 BTree    | BTree  | 1.22        | 0.60          | 2010  |

### 与原生 czdb 库对比

| 序号 | 名称               | 模式    | 总时间 (ms) | 平均时间 (us) | 查询数 |
|-----|--------------------|--------|-------------|---------------|-------|
| 1   | czdb-rs (WASM)     | Memory | 8.75        | 1.60          | 5485  |
| 2   | 原生 czdb          | Memory | 26.66       | 4.90          | 5485  |

**czdb-rs 在 WASM 模式下比原生 czdb 库快 3 倍!**

**性能提示：**
- 请尽量使用单例来查询，避免每次查询都初始化 DbSearcher，这会带来性能瓶颈。
- BTree 模式查询时不是线程安全的，请避免在多线程环境下的并发访问。

## 安装

### Rust

在 `Cargo.toml` 中添加：

```toml
[dependencies]
czdb-rs = "0.1.1"
```
或者

```bash
cargo add czdb-rs
```

### Node.js (WASM)

```bash
npm install czdb-rs
# 或
bun add czdb-rs
```

## 快速开始

### Rust

```rust
use czdb_rs::searcher::{DbSearcher, SearchMode};

fn main() {
    let key = "YOUR_CZDB_KEY";

    // 创建搜索器（默认 Memory 模式 - 最快）
    let db_data = std::fs::read("cz88_public_v4.czdb").expect("读取数据库失败");
    let searcher = DbSearcher::new(db_data, key).expect("初始化搜索器失败");

    let ip = "8.8.8.8";
    match searcher.search(ip) {
        Ok(region) => println!("{}: {}", ip, region),
        Err(e) => println!("错误: {}", e),
    }
}
```

### JavaScript / TypeScript

```typescript
import { CzdbSearcher } from 'czdb-rs';
import * as fs from 'fs';

const key = "YOUR_CZDB_KEY";
const dbData = fs.readFileSync("cz88_public_v4.czdb");

// Memory 模式（默认，最快）
const searcher = new CzdbSearcher(dbData, key);

// 或 BTree 模式：0 = Memory，1 = BTree
const searcherBTree = new CzdbSearcher(dbData, key, 1);

const ip = "8.8.8.8";
const region = searcher.search(ip);
console.log(`${ip}: ${region}`);
```

## 配置

### 构造函数参数

| 参数 | 说明 |
|-----|------|
| `data` | 数据库文件内容 (Uint8Array/Vec<u8>) |
| `key` | 加密密钥 |
| `mode` | 搜索模式（可选）：0 = Memory，1 = BTree |

数据库文件和密钥可从 [www.cz88.net](https://www.cz88.net) 获取。

## 模式选择

**批量查询：** 建议使用 Memory 模式。Memory 模式会将整个数据库加载到内存中，从而在处理大量查询时显著提高查询速度。虽然会增加内存使用，但能大幅提升批量处理效率。

**少量查询：** 如果每个请求只查询少量 IP 地址，使用 BTree 模式更合适。BTree 模式不需要将整个数据库加载到内存中，适用于处理少量查询请求，可减少内存使用，同时保持良好的查询性能。

## 基准测试

### Rust

```bash
# Release 模式基准测试
cargo test --release --test bench_rust -- --nocapture
```

### Node.js

```bash
# 安装依赖
cd npm-test
bun install

# 运行基准测试
CZDB_SECRET=your_key bun run bench.ts
```

## 测试

```bash
# 所有测试
cargo test

# 功能测试
cargo test --test test_search

# Rust 基准测试
cargo test --release --test bench_rust -- --nocapture

# Node.js 基准测试
cd npm-test && CZDB_SECRET=your_key bun run bench.ts
```

## 许可证

Apache-2.0 许可证 - 详情请查看 LICENSE 文件。
