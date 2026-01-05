# czdb-rs

CZDB 搜索器的高性能 Rust 实现，支持 WASM。

[English Documentation](README.md)

## 安装

### Rust

在 `Cargo.toml` 中添加：

```toml
[dependencies]
czdb-rs = "0.1.0"
```

### Node.js (WASM)

```bash
npm install czdb-rs
```

## 使用方法

### Rust

```rust
use czdb_rs::searcher::DbSearcher;
use std::fs;

fn main() {
    let key = "YOUR_CZDB_KEY";

    // IPv4 搜索
    let db_data_v4 = fs::read("cz88_public_v4.czdb").expect("Failed to read IPv4 DB");
    let searcher_v4 = DbSearcher::new(db_data_v4, key).expect("Failed to init IPv4 searcher");
    
    let ip_v4 = "1.1.1.1";
    match searcher_v4.search(ip_v4) {
        Ok(region) => println!("{}: {}", ip_v4, region),
        Err(e) => println!("Error: {}", e),
    }

    // IPv6 搜索
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

// IPv4 搜索
const dbDataV4 = fs.readFileSync("cz88_public_v4.czdb");
const searcherV4 = new CzdbSearcher(dbDataV4, key);

const ipV4 = "8.8.8.8";
try {
    const region = searcherV4.search(ipV4);
    console.log(`${ipV4}: ${region}`);
} catch (e) {
    console.error(e);
}
searcherV4.free(); // 使用完毕后清理内存

// IPv6 搜索
const dbDataV6 = fs.readFileSync("cz88_public_v6.czdb");
const searcherV6 = new CzdbSearcher(dbDataV6, key);

const ipV6 = "2400:3200::1";
try {
    const region = searcherV6.search(ipV6);
    console.log(`${ipV6}: ${region}`);
} catch (e) {
    console.error(e);
}
searcherV6.free(); // 使用完毕后清理内存
```

## 性能测试

### Rust 性能测试汇总

| | name | totalTime(ms) | avgTime(ms) | count | outputFile |
|---|---|---|---|---|---|
| 0 | Rust WASM IPv4 | 1.41 | 0.0003 | 5485 | tests/output/rust_wasm_ipv4.txt |
| 1 | Rust WASM IPv6 | 0.50 | 0.0002 | 2010 | tests/output/rust_wasm_ipv6.txt |

### JS 性能测试汇总

| | name | totalTime | avgTime | count | outputFile |
|---|---|---|---|---|---|
| 0 | Rust WASM IPv4 | 8.73 | 0.0016 | 5485 | tests/output/js_wasm_ipv4.txt |
| 1 | Rust WASM IPv6 | 1.93 | 0.001 | 2010 | tests/output/js_wasm_ipv6.txt |
| 2 | czdb (MEMORY) IPv4 | 23.42 | 0.0043 | 5485 | tests/output/czdb-node_memory_ipv4.txt |
| 3 | czdb (BTREE) IPv4 | 55.02 | 0.01 | 5485 | tests/output/czdb-node_btree_ipv4.txt |
| 4 | czdb (MEMORY) IPv6 | 13.26 | 0.0066 | 2010 | tests/output/czdb-node_memory_ipv6.txt |
| 5 | czdb (BTREE) IPv6 | 34.4 | 0.0171 | 2010 | tests/output/czdb-node_btree_ipv6.txt |
## 免责声明

在正式使用前，请您知晓如下注意事项:

1. 纯真社区版IP库离线版免费提供，并非商业数据库。我们不对该数据库承担任何除服务可用性外的责任。因该IP库的数据准确性等所造成问题，我们不承担任何责任。请您谨慎选择使用。
2. 纯真将定期查询您提交的展示纯真信息的网页或者APP页面。若发现相关页面已经失效或者修改为无关信息，我们有权停止您使用纯真社区版IP库更新服务。
3. 纯真社区版IP库仅授权给您使用，每个授权用户均有自己独特的下载链接，若发现您擅自公开、转让该链接给其他方使用，我们有权停止您使用纯真社区版IP库更新服务，并追究您的相关责任。

测试用的ip数据来源是[╃苍狼山庄╃ ISP IP](https://ispip.clang.cn/)。

## 许可证

Apache-2.0
