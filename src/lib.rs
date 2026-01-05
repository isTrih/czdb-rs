mod decrypt;
pub mod searcher;

use wasm_bindgen::prelude::*;
use crate::searcher::DbSearcher;

#[wasm_bindgen]
pub struct CzdbSearcher {
    inner: DbSearcher,
}

#[wasm_bindgen]
impl CzdbSearcher {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8], key: &str) -> Result<CzdbSearcher, JsError> {
        // For WASM, we copy the data into the struct.
        // data is passed as Uint8Array from JS.
        let searcher = DbSearcher::new(data.to_vec(), key)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(CzdbSearcher { inner: searcher })
    }

    pub fn search(&self, ip: &str) -> Result<String, JsError> {
        self.inner.search(ip).map_err(|e| JsError::new(&e.to_string()))
    }

    // Experimental: Batch search to reduce WASM call overhead
    pub fn search_batch(&self, ips: Vec<String>) -> Result<Vec<String>, JsError> {
        let mut results = Vec::with_capacity(ips.len());
        for ip in ips {
            results.push(self.inner.search(&ip).unwrap_or_else(|_| "Error".to_string()));
        }
        Ok(results)
    }
}
