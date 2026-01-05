use czdb_rs::searcher::DbSearcher;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;

struct BenchResult {
    name: String,
    total_time_ms: f64,
    avg_time_ms: f64,
    count: usize,
    output_file: String,
}

fn run_benchmark(name: &str, db_path: &str, input_path: &str, output_path: &str) -> BenchResult {
    let key = env::var("CZDB_SECRET").expect("CZDB_SECRET not set");
    let data = fs::read(db_path).expect("Failed to read DB file");
    let searcher = DbSearcher::new(data, &key).expect("Failed to init searcher");
    
    let file = fs::File::open(input_path).expect("Failed to open input file");
    let reader = BufReader::new(file);
    
    let ips: Vec<String> = reader.lines()
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
        total_time_ms,
        avg_time_ms,
        count,
        output_file: output_path.to_string(),
    }
}

fn print_table(results: &[BenchResult]) {
    println!("\n=== Benchmark Summary ===");
    println!("┌───┬────────────────────┬───────────────┬───────────────┬───────┬─────────────────────────────┐");
    println!("│   │ name               │ totalTime(ms) │ avgTime(ms)   │ count │ outputFile                  │");
    println!("├───┼────────────────────┼───────────────┼───────────────┼───────┼─────────────────────────────┤");
    
    for (i, res) in results.iter().enumerate() {
        let time_str = format!("{:.2}", res.total_time_ms);
        let avg_str = format!("{:.4}", res.avg_time_ms);
        
        println!("│{:<3}│{:<20}│{:<15}│{:<15}│{:<7}│{:<29}│", 
            format!(" {}", i), 
            format!(" {}", res.name), 
            format!(" {}", time_str), 
            format!(" {}", avg_str), 
            format!(" {}", res.count), 
            format!(" {}", res.output_file)
        );
    }
    
    println!("└───┴────────────────────┴───────────────┴───────────────┴───────┴─────────────────────────────┘");
}

#[test]
fn bench_all() {
    let mut results = Vec::new();
    
    results.push(run_benchmark(
        "Rust WASM IPv4", 
        "czdb/cz88_public_v4.czdb", 
        "tests/IPV4.txt", 
        "tests/output/rust_wasm_ipv4.txt"
    ));
    
    results.push(run_benchmark(
        "Rust WASM IPv6", 
        "czdb/cz88_public_v6.czdb", 
        "tests/IPV6.txt", 
        "tests/output/rust_wasm_ipv6.txt"
    ));
    
    print_table(&results);
}
