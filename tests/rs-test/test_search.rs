use czdb_rs::searcher::DbSearcher;
use std::env;
use std::fs;
use std::path::Path;

#[test]
fn test_ipv4_search() {
    let key = env::var("CZDB_SECRET").expect("CZDB_SECRET not set");
    let db_path = Path::new("czdb/cz88_public_v4.czdb");
    let data = fs::read(db_path).expect("Failed to read DB file");
    
    let searcher = DbSearcher::new(data, &key).expect("Failed to init searcher");
    
    // Test some IPs
    let result = searcher.search("8.8.8.8").expect("Search failed");
    println!("8.8.8.8: {}", result);
    assert!(!result.is_empty());
    
    let result = searcher.search("1.1.1.1").expect("Search failed");
    println!("1.1.1.1: {}", result);
    assert!(!result.is_empty());
}

#[test]
fn test_ipv6_search() {
    let key = env::var("CZDB_SECRET").expect("CZDB_SECRET not set");
    let db_path = Path::new("czdb/cz88_public_v6.czdb");
    let data = fs::read(db_path).expect("Failed to read DB file");
    
    let searcher = DbSearcher::new(data, &key).expect("Failed to init searcher");
    
    // Test some IPs
    let result = searcher.search("2001:4860:4860::8888").expect("Search failed");
    println!("2001:4860:4860::8888: {}", result);
    assert!(!result.is_empty());
}
