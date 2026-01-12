use czdb_rs::searcher::{DbSearcher, SearchMode};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;

struct BenchResult {
    name: String,
    mode: String,
    total_time_ms: f64,
    avg_time_ms: f64,
    count: usize,
    output_file: String,
}

fn run_benchmark_mode(
    name: &str,
    mode: SearchMode,
    db_path: &str,
    input_path: &str,
    output_path: &str,
) -> BenchResult {
    let key = std::env::var("CZDB_SECRET")
        .unwrap_or_else(|_| "YOUR_SECRET_KEY_HERE".to_string());

    let data = fs::read(db_path).expect("Failed to read DB file");
    let searcher = DbSearcher::with_mode(data, &key, mode).expect("Failed to init searcher");

    let file = fs::File::open(input_path).expect("Failed to open input file");
    let reader = BufReader::new(file);

    let ips: Vec<String> = reader
        .lines()
        .map(|l| l.unwrap())
        .map(|l| {
            if let Some(idx) = l.find('/') {
                l[..idx].trim().to_string()
            } else {
                l.trim().to_string()
            }
        })
        .filter(|l| !l.is_empty())
        .collect();

    let count = ips.len();

    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let output_file = fs::File::create(output_path).expect("Failed to create output file");
    let mut writer = BufWriter::new(output_file);

    let start = Instant::now();
    for ip in &ips {
        let result = searcher.search(ip).unwrap_or_else(|_| "Error".to_string());
        writeln!(writer, "{}\t{}", ip, result).expect("Failed to write result");
    }
    let duration = start.elapsed();

    let total_time_ms = duration.as_secs_f64() * 1000.0;
    let avg_time_ms = total_time_ms / count as f64;

    BenchResult {
        name: name.to_string(),
        mode: format!("{:?}", mode),
        total_time_ms,
        avg_time_ms,
        count,
        output_file: output_path.to_string(),
    }
}

fn print_table(results: &[BenchResult]) {
    println!("\n=== Benchmark Summary ===");
    println!(
        "┌─────┬────────────────────────┬────────────┬───────────────────┬───────────────────┬─────────┬──────────────────────────────┐"
    );
    println!(
        "│ No. │ Name                   │ Mode       │ Total Time (ms)   │ Avg Time (ms)     │ Count   │ Output File                 │"
    );
    println!(
        "├─────┼────────────────────────┼────────────┼───────────────────┼───────────────────┼─────────┼──────────────────────────────┤"
    );

    for (i, res) in results.iter().enumerate() {
        let time_str = format!("{:.2}", res.total_time_ms);
        let avg_str = format!("{:.4}", res.avg_time_ms);

        println!(
            "│{:^5}│{:^24}│{:^12}│{:^19}│{:^19}│{:^9}│{:^30}│",
            i + 1,
            res.name,
            res.mode,
            time_str,
            avg_str,
            res.count,
            res.output_file
        );
    }

    println!(
        "└─────┴────────────────────────┴────────────┴───────────────────┴───────────────────┴─────────┴──────────────────────────────┘"
    );
}

/// Benchmark for IPv4 Memory mode
#[test]
fn bench_ipv4_memory() {
    let result = run_benchmark_mode(
        "Rust IPv4",
        SearchMode::Memory,
        "czdb/cz88_public_v4.czdb",
        "tests/IPV4.txt",
        "tests/output/rust_ipv4_memory.txt",
    );
    println!(
        "IPv4 Memory: {:.4} ms avg, {} queries",
        result.avg_time_ms, result.count
    );
}

/// Benchmark for IPv4 BTree mode
#[test]
fn bench_ipv4_btree() {
    let result = run_benchmark_mode(
        "Rust IPv4 BTree",
        SearchMode::BTree,
        "czdb/cz88_public_v4.czdb",
        "tests/IPV4.txt",
        "tests/output/rust_ipv4_btree.txt",
    );
    println!(
        "IPv4 BTree: {:.4} ms avg, {} queries",
        result.avg_time_ms, result.count
    );
}

/// Benchmark for IPv6 Memory mode
#[test]
fn bench_ipv6_memory() {
    let result = run_benchmark_mode(
        "Rust IPv6",
        SearchMode::Memory,
        "czdb/cz88_public_v6.czdb",
        "tests/IPV6.txt",
        "tests/output/rust_ipv6_memory.txt",
    );
    println!(
        "IPv6 Memory: {:.4} ms avg, {} queries",
        result.avg_time_ms, result.count
    );
}

/// Benchmark for IPv6 BTree mode
#[test]
fn bench_ipv6_btree() {
    let result = run_benchmark_mode(
        "Rust IPv6 BTree",
        SearchMode::BTree,
        "czdb/cz88_public_v6.czdb",
        "tests/IPV6.txt",
        "tests/output/rust_ipv6_btree.txt",
    );
    println!(
        "IPv6 BTree: {:.4} ms avg, {} queries",
        result.avg_time_ms, result.count
    );
}

/// Run all benchmarks and print comparison table
#[test]
fn bench_all_modes() {
    let mut results = Vec::new();

    // IPv4 benchmarks
    results.push(run_benchmark_mode(
        "Rust IPv4",
        SearchMode::Memory,
        "czdb/cz88_public_v4.czdb",
        "tests/IPV4.txt",
        "tests/output/rust_ipv4_memory.txt",
    ));

    results.push(run_benchmark_mode(
        "Rust IPv4 BTree",
        SearchMode::BTree,
        "czdb/cz88_public_v4.czdb",
        "tests/IPV4.txt",
        "tests/output/rust_ipv4_btree.txt",
    ));

    // IPv6 benchmarks
    results.push(run_benchmark_mode(
        "Rust IPv6",
        SearchMode::Memory,
        "czdb/cz88_public_v6.czdb",
        "tests/IPV6.txt",
        "tests/output/rust_ipv6_memory.txt",
    ));

    results.push(run_benchmark_mode(
        "Rust IPv6 BTree",
        SearchMode::BTree,
        "czdb/cz88_public_v6.czdb",
        "tests/IPV6.txt",
        "tests/output/rust_ipv6_btree.txt",
    ));

    print_table(&results);
}

/// Quick single-mode benchmark for development
#[test]
fn bench_quick() {
    let key = std::env::var("CZDB_SECRET")
        .unwrap_or_else(|_| "YOUR_SECRET_KEY_HERE".to_string());

    let data = fs::read("czdb/cz88_public_v4.czdb").expect("Failed to read DB file");
    let searcher = DbSearcher::with_mode(data, &key, SearchMode::Memory)
        .expect("Failed to init searcher");

    let ips = ["8.8.8.8", "1.1.1.1", "223.5.5.5", "119.29.29.29"];

    let start = Instant::now();
    for _ in 0..10000 {
        for ip in &ips {
            let _ = searcher.search(ip);
        }
    }
    let duration = start.elapsed();

    println!("Quick benchmark: {} ms for 40000 queries", duration.as_secs_f64() * 1000.0);
}
