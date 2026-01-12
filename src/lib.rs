mod decrypt;
pub mod searcher;

use wasm_bindgen::prelude::*;
use crate::searcher::{DbSearcher, SearchMode};

#[wasm_bindgen]
pub struct CzdbSearcher {
    inner: DbSearcher,
}

#[wasm_bindgen]
impl CzdbSearcher {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8], key: &str) -> Result<CzdbSearcher, JsError> {
        let searcher = DbSearcher::new(data.to_vec(), key)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(CzdbSearcher { inner: searcher })
    }

    /// Create with specific search mode (0=Memory, 1=BTree)
    #[wasm_bindgen]
    pub fn new_with_mode(data: &[u8], key: &str, mode: u8) -> Result<CzdbSearcher, JsError> {
        let search_mode = match mode {
            0 => SearchMode::Memory,
            1 => SearchMode::BTree,
            _ => SearchMode::Memory,
        };
        let searcher = DbSearcher::with_mode(data.to_vec(), key, search_mode)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(CzdbSearcher { inner: searcher })
    }

    pub fn search(&self, ip: &str) -> Result<String, JsError> {
        self.inner.search(ip).map_err(|e| JsError::new(&e.to_string()))
    }

    // Batch search to reduce WASM call overhead
    pub fn search_batch(&self, ips: Vec<String>) -> Result<Vec<String>, JsError> {
        let mut results = Vec::with_capacity(ips.len());
        for ip in ips {
            results.push(self.inner.search(&ip).unwrap_or_else(|_| "Error".to_string()));
        }
        Ok(results)
    }

    /// Get current search mode (0=Memory, 1=BTree)
    pub fn search_mode(&self) -> u8 {
        match self.inner.search_mode() {
            SearchMode::Memory => 0,
            SearchMode::BTree => 1,
        }
    }
}
