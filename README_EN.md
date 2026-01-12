# czdb-rs
<p align="center">English Documentation | <a href="./README.md">中文文档</a></p>

A high-performance Rust implementation of CZDB IP geolocation searcher with WASM support.

## Features

- IP geolocation lookup
- Supports IPv4 and IPv6 addresses
- Simple and easy-to-use API
- Two search modes: Memory (fastest) and BTree (low memory)

## Performance

### Rust (Native)

**Release mode benchmarks (5485 IPv4 / 2010 IPv6 queries):**

| No. | Name               | Mode   | Total Time (ms) | Avg Time (ms) | Count |
|-----|--------------------|--------|-----------------|---------------|-------|
| 1   | Rust IPv4          | Memory | 1.46            | 0.0003        | 5485  |
| 2   | Rust IPv4 BTree    | BTree  | 1.65            | 0.0003        | 5485  |
| 3   | Rust IPv6          | Memory | 0.43            | 0.0002        | 2010  |
| 4   | Rust IPv6 BTree    | BTree  | 0.48            | 0.0002        | 2010  |

### Node.js (WASM)

**Benchmark results (5485 IPv4 / 2010 IPv6 queries):**

| No. | Name               | Mode   | Total Time (ms) | Avg Time (us) | Count |
|-----|--------------------|--------|-----------------|---------------|-------|
| 1   | WASM IPv4          | Memory | 8.75            | 1.60          | 5485  |
| 2   | WASM IPv4 BTree    | BTree  | 3.36            | 0.60          | 5485  |
| 3   | WASM IPv6          | Memory | 1.51            | 0.80          | 2010  |
| 4   | WASM IPv6 BTree    | BTree  | 1.22            | 0.60          | 2010  |

### Comparison with Native czdb Library

| No. | Name               | Mode   | Total Time (ms) | Avg Time (us) | Count |
|-----|--------------------|--------|-----------------|---------------|-------|
| 1   | czdb-rs (WASM)     | Memory | 8.75            | 1.60          | 5485  |
| 2   | Native czdb        | Memory | 26.66           | 4.90          | 5485  |

**czdb-rs is 3x faster than the native czdb library in WASM mode!**

**Performance Tips:**
- Use singleton pattern for queries. Initializing DbSearcher for each query will cause performance bottleneck.
- BTree mode is not thread-safe. Avoid concurrent access in multi-threaded environments.

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
czdb-rs = "0.1.1"
```
or
```bash
cargo add czdb-rs
```
### Node.js (WASM)

```bash
npm install czdb-rs
# or
bun add czdb-rs
```

## Quick Start

### Rust

```rust
use czdb_rs::searcher::{DbSearcher, SearchMode};

fn main() {
    let key = "YOUR_CZDB_KEY";

    // Create searcher (default Memory mode - fastest)
    let db_data = std::fs::read("cz88_public_v4.czdb").expect("Failed to read DB");
    let searcher = DbSearcher::new(db_data, key).expect("Failed to init searcher");

    let ip = "8.8.8.8";
    match searcher.search(ip) {
        Ok(region) => println!("{}: {}", ip, region),
        Err(e) => println!("Error: {}", e),
    }
}
```

### JavaScript / TypeScript

```typescript
import { CzdbSearcher } from 'czdb-rs';
import * as fs from 'fs';

const key = "YOUR_CZDB_KEY";
const dbData = fs.readFileSync("cz88_public_v4.czdb");

// Memory mode (default, fastest)
const searcher = new CzdbSearcher(dbData, key);

// Or BTree mode: 0 = Memory, 1 = BTree
const searcherBTree = new CzdbSearcher(dbData, key, 1);

const ip = "8.8.8.8";
const region = searcher.search(ip);
console.log(`${ip}: ${region}`);
```

## Configuration

### Constructor Parameters

| Parameter | Description |
|-----------|-------------|
| `data` | Database file content (Uint8Array/Vec<u8>) |
| `key` | Encryption key |
| `mode` | Search mode (optional): 0 = Memory, 1 = BTree |

Get database files and keys from [www.cz88.net](https://www.cz88.net).

## Mode Selection

**Batch queries:** Use Memory mode. Memory mode loads the entire database into memory, significantly improving query speed for large volumes. Although this increases memory usage, it greatly improves batch processing efficiency.

**Single queries:** Use BTree mode if each request only queries a small number of IP addresses. BTree mode doesn't require loading the entire database into memory, suitable for handling small volumes of queries while reducing memory usage.

## Benchmark

### Rust

```bash
# Release mode benchmark
cargo test --release --test bench_rust -- --nocapture
```

### Node.js

```bash
# Install dependencies
cd npm-test
bun install

# Run benchmark
CZDB_SECRET=your_key bun run bench.ts
```

## Testing

```bash
# All tests
cargo test

# Functional tests
cargo test --test test_search

# Rust benchmark
cargo test --release --test bench_rust -- --nocapture

# Node.js benchmark
cd npm-test && CZDB_SECRET=your_key bun run bench.ts
```

## License

Licensed under Apache-2.0 - see LICENSE file for details.
