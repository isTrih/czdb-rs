# czdb-rs

A high-performance Rust implementation of the CZDB searcher with WASM support.

[中文文档](README_CN.md)

## Installation

### Rust

Add this to your `Cargo.toml`:

```toml
[dependencies]
czdb-rs = "0.1.0"
```

### Node.js (WASM)

```bash
npm install czdb-rs
```

## Usage

### Rust

```rust
use czdb_rs::searcher::DbSearcher;
use std::fs;

fn main() {
    let key = "YOUR_CZDB_KEY";

    // IPv4 Search
    let db_data_v4 = fs::read("cz88_public_v4.czdb").expect("Failed to read IPv4 DB");
    let searcher_v4 = DbSearcher::new(db_data_v4, key).expect("Failed to init IPv4 searcher");
    
    let ip_v4 = "1.1.1.1";
    match searcher_v4.search(ip_v4) {
        Ok(region) => println!("{}: {}", ip_v4, region),
        Err(e) => println!("Error: {}", e),
    }

    // IPv6 Search
    let db_data_v6 = fs::read("cz88_public_v6.czdb").expect("Failed to read IPv6 DB");
    let searcher_v6 = DbSearcher::new(db_data_v6, key).expect("Failed to init IPv6 searcher");

    let ip_v6 = "2001:4860:4860::8888";
    match searcher_v6.search(ip_v6) {
        Ok(region) => println!("{}: {}", ip_v6, region),
        Err(e) => println!("Error: {}", e),
    }
}
```

### JavaScript / TypeScript

```typescript
import { CzdbSearcher } from 'czdb-rs';
import * as fs from 'fs';

const key = "YOUR_CZDB_KEY";

// IPv4 Search
const dbDataV4 = fs.readFileSync("cz88_public_v4.czdb");
const searcherV4 = new CzdbSearcher(dbDataV4, key);

const ipV4 = "8.8.8.8";
try {
    const region = searcherV4.search(ipV4);
    console.log(`${ipV4}: ${region}`);
} catch (e) {
    console.error(e);
}
searcherV4.free(); // Clean up memory

// IPv6 Search
const dbDataV6 = fs.readFileSync("cz88_public_v6.czdb");
const searcherV6 = new CzdbSearcher(dbDataV6, key);

const ipV6 = "2400:3200::1";
try {
    const region = searcherV6.search(ipV6);
    console.log(`${ipV6}: ${region}`);
} catch (e) {
    console.error(e);
}
searcherV6.free(); // Clean up memory
```

## Benchmark

### Rust Benchmark Summary

| | name | totalTime(ms) | avgTime(ms) | count | outputFile |
|---|---|---|---|---|---|
| 0 | Rust WASM IPv4 | 1.41 | 0.0003 | 5485 | tests/output/rust_wasm_ipv4.txt |
| 1 | Rust WASM IPv6 | 0.50 | 0.0002 | 2010 | tests/output/rust_wasm_ipv6.txt |

### JS Benchmark Summary

| | name | totalTime | avgTime | count | outputFile |
|---|---|---|---|---|---|
| 0 | Rust WASM IPv4 | 8.73 | 0.0016 | 5485 | tests/output/js_wasm_ipv4.txt |
| 1 | Rust WASM IPv6 | 1.93 | 0.001 | 2010 | tests/output/js_wasm_ipv6.txt |
| 2 | czdb (MEMORY) IPv4 | 23.42 | 0.0043 | 5485 | tests/output/czdb-node_memory_ipv4.txt |
| 3 | czdb (BTREE) IPv4 | 55.02 | 0.01 | 5485 | tests/output/czdb-node_btree_ipv4.txt |
| 4 | czdb (MEMORY) IPv6 | 13.26 | 0.0066 | 2010 | tests/output/czdb-node_memory_ipv6.txt |
| 5 | czdb (BTREE) IPv6 | 34.4 | 0.0171 | 2010 | tests/output/czdb-node_btree_ipv6.txt |

## Disclaimer

Before using, please be aware of the following notices:

1. The Pure IP Community Edition offline library is provided for free and is not a commercial database. We assume no responsibility for this database other than service availability. We are not responsible for any issues caused by the accuracy of the IP library data. Please use it with caution.
2. Pure IP will periodically check the web pages or APP pages where you display Pure IP information. If we find that the relevant pages are invalid or modified to irrelevant information, we have the right to stop your service for updating the Pure IP Community Edition IP library.
3. The Pure IP Community Edition IP library is authorized only for your use. Each authorized user has a unique download link. If we find that you have unauthorizedly disclosed or transferred the link to others, we have the right to stop your service for updating the Pure IP Community Edition IP library and hold you accountable.

The IP data used for testing comes from [╃Clang ISP IP╃](https://ispip.clang.cn/).

## License

Apache-2.0
