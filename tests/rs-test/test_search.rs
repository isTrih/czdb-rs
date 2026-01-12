use czdb_rs::searcher::{DbSearcher, SearchMode};
use std::fs;
use std::path::Path;

/// Test IPv4 search with Memory mode (default)
#[test]
fn test_ipv4_search_memory() {
    let key = std::env::var("CZDB_SECRET")
        .unwrap_or_else(|_| "YOUR_SECRET_KEY_HERE".to_string());

    let db_path = Path::new("czdb/cz88_public_v4.czdb");
    let data = fs::read(db_path).expect("Failed to read DB file");

    let searcher = DbSearcher::new(data, &key).expect("Failed to init searcher");

    let result = searcher.search("8.8.8.8").expect("Search failed");
    println!("8.8.8.8 (Memory): {}", result);
    assert!(!result.is_empty());

    let result = searcher.search("1.1.1.1").expect("Search failed");
    println!("1.1.1.1 (Memory): {}", result);
    assert!(!result.is_empty());
}

/// Test IPv6 search with Memory mode
#[test]
fn test_ipv6_search_memory() {
    let key = std::env::var("CZDB_SECRET")
        .unwrap_or_else(|_| "YOUR_SECRET_KEY_HERE".to_string());

    let db_path = Path::new("czdb/cz88_public_v6.czdb");
    let data = fs::read(db_path).expect("Failed to read DB file");

    let searcher = DbSearcher::new(data, &key).expect("Failed to init searcher");

    let result = searcher.search("2001:4860:4860::8888").expect("Search failed");
    println!("2001:4860:4860::8888 (Memory): {}", result);
    assert!(!result.is_empty());
}

/// Test IPv4 search with BTree mode
#[test]
fn test_ipv4_search_btree() {
    let key = std::env::var("CZDB_SECRET")
        .unwrap_or_else(|_| "YOUR_SECRET_KEY_HERE".to_string());

    let db_path = Path::new("czdb/cz88_public_v4.czdb");
    let data = fs::read(db_path).expect("Failed to read DB file");

    let searcher = DbSearcher::with_mode(data, &key, SearchMode::BTree)
        .expect("Failed to init BTree searcher");

    let result = searcher.search("8.8.8.8").expect("BTree search failed");
    println!("8.8.8.8 (BTree): {}", result);
    assert!(!result.is_empty());
}

/// Test IPv6 search with BTree mode
#[test]
fn test_ipv6_search_btree() {
    let key = std::env::var("CZDB_SECRET")
        .unwrap_or_else(|_| "YOUR_SECRET_KEY_HERE".to_string());

    let db_path = Path::new("czdb/cz88_public_v6.czdb");
    let data = fs::read(db_path).expect("Failed to read DB file");

    let searcher = DbSearcher::with_mode(data, &key, SearchMode::BTree)
        .expect("Failed to init BTree searcher");

    let result = searcher.search("2001:4860:4860::8888").expect("BTree search failed");
    println!("2001:4860:4860::8888 (BTree): {}", result);
    assert!(!result.is_empty());
}

/// Test that both modes return consistent results
#[test]
fn test_modes_consistent() {
    let key = std::env::var("CZDB_SECRET")
        .unwrap_or_else(|_| "YOUR_SECRET_KEY_HERE".to_string());

    let db_path_v4 = Path::new("czdb/cz88_public_v4.czdb");
    let data_v4 = fs::read(db_path_v4).expect("Failed to read DB file");

    let test_ips = ["8.8.8.8", "1.1.1.1", "192.168.1.1", "223.5.5.5"];

    for ip in &test_ips {
        let memory_result = DbSearcher::with_mode(data_v4.clone(), &key, SearchMode::Memory)
            .expect("Failed to init Memory searcher")
            .search(ip)
            .expect("Memory search failed");

        let btree_result = DbSearcher::with_mode(data_v4.clone(), &key, SearchMode::BTree)
            .expect("Failed to init BTree searcher")
            .search(ip)
            .expect("BTree search failed");

        println!("{}: Memory={}, BTree={}", ip, memory_result, btree_result);

        assert_eq!(memory_result, btree_result, "Memory and BTree results differ for {}", ip);
    }
}
