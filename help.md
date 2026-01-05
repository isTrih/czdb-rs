# 帮助文档

本文档介绍如何构建、发布和使用 `czdb-rs` 项目。

## 1. 常用命令 (Makefile)

本项目提供了 `Makefile` 来简化常用操作。

*   **构建 WASM 包**: `make build-wasm`
*   **运行 Rust 测试**: `make test-rs` (需要设置 `CZDB_SECRET`)
*   **运行 Rust 性能测试**: `make bench-rs` (需要设置 `CZDB_SECRET`)
*   **运行 JS/WASM 性能测试**: `make test-js` (需要设置 `CZDB_SECRET`)
*   **运行所有测试**: `make test-all` (需要设置 `CZDB_SECRET`)
*   **清理构建产物**: `make clean`

## 2. 构建项目

### 2.1 构建 Rust 项目

```bash
cargo build --release
```

### 2.2 构建 WASM (Node.js)

```bash
make build-wasm
```

## 3. 发布到 NPM

使用 `make build-wasm` 构建后，进入 `pkg` 目录发布：

```bash
make build-wasm
cd pkg
npm publish
```

## 4. 发布到 Crates.io

发布到 Rust 官方包仓库 crates.io：

1. 确保已登录：
   ```bash
   cargo login
   ```
2. 发布：
   ```bash
   cargo publish
   ```

## 5. 运行测试

确保已设置环境变量 `CZDB_SECRET`。

### 5.1 Rust 测试

```bash
export CZDB_SECRET="your_secret_key"
make test-rs
```

### 5.2 Rust 性能测试

```bash
export CZDB_SECRET="your_secret_key"
make bench-rs
```

### 5.3 JS/WASM 性能测试

```bash
export CZDB_SECRET="your_secret_key"
make test-js
```

