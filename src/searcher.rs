//! CZDB Searcher with Memory and BTree modes
//!
//! Supported search modes:
//! - Memory: Full memory load with optimized binary search
//! - BTree: Hierarchical index, file streaming (no full load)

use crate::decrypt::{decrypt_aes_ecb, decrypt_xor};
use byteorder::{ByteOrder, LE};
use std::net::IpAddr;
use std::str::FromStr;
use thiserror::Error;
use std::io::Cursor;

#[derive(Error, Debug)]
pub enum CzdbError {
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("Decryption error")]
    DecryptError(#[from] crate::decrypt::DecryptError),
    #[error("Invalid database format")]
    InvalidFormat,
    #[allow(dead_code)]
    #[error("Database expired")]
    Expired,
    #[error("Client ID mismatch")]
    ClientIdMismatch,
    #[error("IP parse error")]
    IpParseError(#[from] std::net::AddrParseError),
    #[error("Msgpack decode error")]
    MsgpackError(#[from] rmp::decode::ValueReadError),
    #[error("Invalid IP Type")]
    InvalidIpType,
    #[error("Invalid search mode")]
    InvalidSearchMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IpType {
    Ipv4,
    Ipv6,
}

/// Search mode enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchMode {
    /// Full memory load with optimized binary search
    Memory,
    /// Hierarchical index, file streaming (no full load)
    BTree,
}

/// Header block for BTree mode (16 bytes IP + 4 bytes pointer)
const HEADER_BLOCK_LENGTH: usize = 20;
const SUPER_PART_LENGTH: usize = 17;

/// BTree mode header index
#[derive(Debug, Clone)]
struct BTreeHeader {
    header_sip: Vec<Vec<u8>>,  // Start IPs for each block
    header_ptr: Vec<usize>,    // Pointers to each block
}

/// Main searcher with Memory and BTree modes
pub struct DbSearcher {
    // Common fields
    data: Vec<u8>,              // Database data
    start_offset: usize,        // Data start offset
    index_start_offset: usize,  // Index start offset
    ip_type: IpType,
    ip_bytes_len: usize,
    column_selection: u32,
    geo_map_data: Option<Vec<u8>>,

    // Mode-specific fields
    search_mode: SearchMode,

    // Memory mode: flat index arrays
    // Store raw index data for cache-friendly access
    index_data: Vec<u8>,        // Raw index bytes
    index_v4_keys: Vec<u32>,    // IPv4 start IPs for binary search
    index_v6_keys: Vec<u128>,   // IPv6 start IPs for binary search
    record_len: usize,          // Length of each index record

    // BTree mode: hierarchical index
    btree_header: Option<BTreeHeader>,
    end_index_ptr: usize,
}

impl DbSearcher {
    /// Create a new searcher with default mode (Memory)
    pub fn new(data: Vec<u8>, key: &str) -> Result<Self, CzdbError> {
        Self::with_mode(data, key, SearchMode::Memory)
    }

    /// Create a searcher with specific mode
    pub fn with_mode(data: Vec<u8>, key: &str, mode: SearchMode) -> Result<Self, CzdbError> {
        let (_header_block, offset) = Self::parse_header(&data, key)?;

        // Read Super Header (17 bytes) at offset
        if data.len() < offset + SUPER_PART_LENGTH {
            return Err(CzdbError::InvalidFormat);
        }
        let super_header = &data[offset..offset + SUPER_PART_LENGTH];

        let ip_type = if super_header[0] & 1 == 0 { IpType::Ipv4 } else { IpType::Ipv6 };
        let header_block_size = LE::read_u32(&super_header[5..9]) as usize;
        let start_index_ptr = header_block_size;
        let end_index_ptr = LE::read_u32(&super_header[13..17]) as usize;
        let ip_bytes_len = if ip_type == IpType::Ipv4 { 4 } else { 16 };
        let record_len = if ip_type == IpType::Ipv4 { 13 } else { 37 };

        let mut searcher = DbSearcher {
            data: data.clone(),
            start_offset: offset,
            index_start_offset: offset + start_index_ptr,
            ip_type,
            ip_bytes_len,
            column_selection: 0,
            geo_map_data: None,
            search_mode: mode,
            index_data: Vec::new(),
            index_v4_keys: Vec::new(),
            index_v6_keys: Vec::new(),
            record_len,
            btree_header: None,
            end_index_ptr,
        };

        // Load geo mapping first (needed by all modes)
        searcher.load_geo_mapping(key, &data)?;

        // Build index based on mode
        match mode {
            SearchMode::Memory => {
                searcher.build_memory_index(start_index_ptr, end_index_ptr, &data)?;
            }
            SearchMode::BTree => {
                searcher.build_btree_index(start_index_ptr, end_index_ptr, &data)?;
            }
        }

        Ok(searcher)
    }

    fn parse_header(data: &[u8], key: &str) -> Result<(HyperHeaderBlock, usize), CzdbError> {
        if data.len() < 12 {
            return Err(CzdbError::InvalidFormat);
        }

        let version = LE::read_u32(&data[0..4]);
        let client_id = LE::read_u32(&data[4..8]);
        let encrypted_block_size = LE::read_u32(&data[8..12]) as usize;

        if data.len() < 12 + encrypted_block_size {
            return Err(CzdbError::InvalidFormat);
        }

        let encrypted_bytes = &data[12..12 + encrypted_block_size];
        let decrypted_bytes = decrypt_aes_ecb(key, encrypted_bytes)?;

        if decrypted_bytes.len() < 8 {
            return Err(CzdbError::InvalidFormat);
        }

        let first_u32 = LE::read_u32(&decrypted_bytes[0..4]);
        let decrypted_client_id = first_u32 >> 20;
        let _expiration_date = first_u32 & 0xFFFFF;
        let random_size = LE::read_u32(&decrypted_bytes[4..8]) as usize;

        if decrypted_client_id != client_id {
            return Err(CzdbError::ClientIdMismatch);
        }

        let header_block = HyperHeaderBlock {
            version,
            client_id,
            encrypted_block_size,
            decrypted_block: DecryptedBlock {
                client_id: decrypted_client_id,
                expiration_date: _expiration_date,
                random_size,
            },
        };

        let offset = 12 + encrypted_block_size + random_size;
        Ok((header_block, offset))
    }

    fn load_geo_mapping(&mut self, key: &str, data: &[u8]) -> Result<(), CzdbError> {
        let column_selection_ptr = self.start_offset + self.end_index_ptr + self.ip_bytes_len * 2 + 5;

        if data.len() < column_selection_ptr + 4 {
            return Ok(());
        }

        self.column_selection = LE::read_u32(&data[column_selection_ptr..column_selection_ptr + 4]);

        if self.column_selection == 0 {
            return Ok(());
        }

        let geo_map_ptr = column_selection_ptr + 4;
        if data.len() < geo_map_ptr + 4 {
            return Err(CzdbError::InvalidFormat);
        }

        let geo_map_size = LE::read_u32(&data[geo_map_ptr..geo_map_ptr + 4]) as usize;
        let geo_map_data_ptr = geo_map_ptr + 4;

        if data.len() < geo_map_data_ptr + geo_map_size {
            return Err(CzdbError::InvalidFormat);
        }

        let mut geo_map_data = data[geo_map_data_ptr..geo_map_data_ptr + geo_map_size].to_vec();
        decrypt_xor(key, &mut geo_map_data)?;

        self.geo_map_data = Some(geo_map_data);

        Ok(())
    }

    /// Build memory index with cache-friendly layout
    fn build_memory_index(&mut self, start_ptr: usize, end_ptr: usize, data: &[u8]) -> Result<(), CzdbError> {
        let start_offset = self.start_offset + start_ptr;
        let end_offset = self.start_offset + end_ptr;

        if end_offset > data.len() {
            return Err(CzdbError::InvalidFormat);
        }

        // Copy raw index data for fast access
        self.index_data = data[start_offset..end_offset].to_vec();

        let count = (end_ptr - start_ptr) / self.record_len + 1;

        match self.ip_type {
            IpType::Ipv4 => {
                self.index_v4_keys.reserve(count);
                let mut ptr = 0;
                while ptr + self.record_len <= self.index_data.len() {
                    let start_ip = u32::from_be_bytes(self.index_data[ptr..ptr+4].try_into().unwrap());
                    self.index_v4_keys.push(start_ip);
                    ptr += self.record_len;
                }
            }
            IpType::Ipv6 => {
                self.index_v6_keys.reserve(count);
                let mut ptr = 0;
                while ptr + self.record_len <= self.index_data.len() {
                    let start_ip = u128::from_be_bytes(self.index_data[ptr..ptr+16].try_into().unwrap());
                    self.index_v6_keys.push(start_ip);
                    ptr += self.record_len;
                }
            }
        }
        Ok(())
    }

    /// Build BTree hierarchical index
    fn build_btree_index(&mut self, start_ptr: usize, end_ptr: usize, data: &[u8]) -> Result<(), CzdbError> {
        // Read total header block size from super header at position 9
        let total_header_block_size = LE::read_u32(&data[self.start_offset + 9..self.start_offset + 13]) as usize;

        // Read the header block data
        let header_data_offset = self.start_offset + SUPER_PART_LENGTH;
        if header_data_offset + total_header_block_size > data.len() {
            return Err(CzdbError::InvalidFormat);
        }

        let header_data = &data[header_data_offset..header_data_offset + total_header_block_size];

        let len = total_header_block_size / HEADER_BLOCK_LENGTH;
        let mut header_sip: Vec<Vec<u8>> = Vec::with_capacity(len);
        let mut header_ptr: Vec<usize> = Vec::with_capacity(len);

        let mut ptr = 0;
        while ptr < total_header_block_size {
            let data_ptr = LE::read_u32(&header_data[ptr + 16..ptr + 20]) as usize;
            if data_ptr == 0 {
                break;
            }

            let sip = header_data[ptr..ptr + 16].to_vec();
            header_sip.push(sip);
            header_ptr.push(data_ptr);
            ptr += HEADER_BLOCK_LENGTH;
        }

        self.btree_header = Some(BTreeHeader {
            header_sip,
            header_ptr,
        });

        Ok(())
    }

    /// Main search interface - dispatches to appropriate mode
    pub fn search(&self, ip: &str) -> Result<String, CzdbError> {
        let ip_addr = IpAddr::from_str(ip)?;

        match (self.ip_type, ip_addr) {
            (IpType::Ipv4, IpAddr::V4(addr)) => self.search_ipv4(addr.octets()),
            (IpType::Ipv6, IpAddr::V6(addr)) => self.search_ipv6(addr.octets()),
            _ => Err(CzdbError::InvalidIpType),
        }
    }

    /// IPv4 search dispatcher
    fn search_ipv4(&self, ip: [u8; 4]) -> Result<String, CzdbError> {
        let ip_u32 = u32::from_be_bytes(ip);

        match self.search_mode {
            SearchMode::Memory => self.memory_search_ipv4(ip_u32),
            SearchMode::BTree => self.btree_search_ipv4(ip),
        }
    }

    /// IPv6 search dispatcher
    fn search_ipv6(&self, ip: [u8; 16]) -> Result<String, CzdbError> {
        let ip_u128 = u128::from_be_bytes(ip);

        match self.search_mode {
            SearchMode::Memory => self.memory_search_ipv6(ip_u128),
            SearchMode::BTree => self.btree_search_ipv6(ip),
        }
    }

    /// Memory mode: Standard binary search with cached index data
    fn memory_search_ipv4(&self, ip: u32) -> Result<String, CzdbError> {
        let idx = match self.index_v4_keys.binary_search(&ip) {
            Ok(i) => i,
            Err(i) => if i > 0 { i - 1 } else { return Ok("Unknown".to_string()) },
        };

        // Read record directly from cached index data
        let offset = idx * self.record_len;

        let end_ip = u32::from_be_bytes(self.index_data[offset+4..offset+8].try_into().unwrap());

        if ip <= end_ip {
            let data_ptr = LE::read_u32(&self.index_data[offset+8..offset+12]);
            let data_len = self.index_data[offset+12];
            return self.get_region(data_ptr as usize, data_len as usize);
        }

        Ok("Unknown".to_string())
    }

    fn memory_search_ipv6(&self, ip: u128) -> Result<String, CzdbError> {
        let idx = match self.index_v6_keys.binary_search(&ip) {
            Ok(i) => i,
            Err(i) => if i > 0 { i - 1 } else { return Ok("Unknown".to_string()) },
        };

        let offset = idx * self.record_len;

        let end_ip = u128::from_be_bytes(self.index_data[offset+16..offset+32].try_into().unwrap());

        if ip <= end_ip {
            let data_ptr = LE::read_u32(&self.index_data[offset+32..offset+36]);
            let data_len = self.index_data[offset+36];
            return self.get_region(data_ptr as usize, data_len as usize);
        }

        Ok("Unknown".to_string())
    }

    /// BTree mode: Hierarchical index search
    fn btree_search_ipv4(&self, ip: [u8; 4]) -> Result<String, CzdbError> {
        let header = self.btree_header.as_ref().ok_or(CzdbError::InvalidSearchMode)?;

        // Binary search on header
        let mut l = 0i32;
        let mut h = (header.header_sip.len() as i32) - 1;
        let mut sptr = 0usize;
        let mut eptr = 0usize;

        while l <= h {
            let m = (l + h) / 2;
            let cmp = Self::compare_ip_bytes(&ip, &header.header_sip[m as usize], 4);

            if cmp < 0 {
                h = m - 1;
            } else if cmp > 0 {
                l = m + 1;
            } else {
                sptr = header.header_ptr[if m > 0 { m as usize - 1 } else { 0 }];
                eptr = header.header_ptr[m as usize];
                break;
            }
        }

        if l > h {
            if l == 0 {
                return Ok("Unknown".to_string());
            }
            if (l as usize) < header.header_sip.len() {
                sptr = header.header_ptr[l as usize - 1];
                eptr = header.header_ptr[l as usize];
            } else if h >= 0 && (h as usize) + 1 < header.header_sip.len() {
                sptr = header.header_ptr[h as usize];
                eptr = header.header_ptr[h as usize + 1];
            } else {
                sptr = header.header_ptr[header.header_sip.len() - 1];
                eptr = sptr + self.record_len;
            }
        }

        if sptr == 0 {
            return Ok("Unknown".to_string());
        }

        // Read index block directly from data (no extra allocation)
        let block_len = eptr - sptr;
        let data_offset = self.start_offset + sptr;

        // Binary search in block
        let mut l = 0i32;
        let mut h = (block_len / self.record_len) as i32 - 1;
        let mut data_ptr = 0u32;
        let mut data_len = 0u8;

        while l <= h {
            let m = (l + h) / 2;
            let p = m as usize * self.record_len;
            let slice_offset = data_offset + p;

            let start_ip = u32::from_be_bytes(self.data[slice_offset..slice_offset+4].try_into().unwrap());
            let end_ip = u32::from_be_bytes(self.data[slice_offset+4..slice_offset+8].try_into().unwrap());

            let ip_u32 = u32::from_be_bytes(ip);

            if ip_u32 >= start_ip && ip_u32 <= end_ip {
                data_ptr = LE::read_u32(&self.data[slice_offset+8..slice_offset+12]);
                data_len = self.data[slice_offset+12];
                break;
            } else if ip_u32 < start_ip {
                h = m - 1;
            } else {
                l = m + 1;
            }
        }

        if data_ptr == 0 {
            return Ok("Unknown".to_string());
        }

        self.get_region(data_ptr as usize, data_len as usize)
    }

    fn btree_search_ipv6(&self, ip: [u8; 16]) -> Result<String, CzdbError> {
        let header = self.btree_header.as_ref().ok_or(CzdbError::InvalidSearchMode)?;

        let mut l = 0i32;
        let mut h = (header.header_sip.len() as i32) - 1;
        let mut sptr = 0usize;
        let mut eptr = 0usize;

        while l <= h {
            let m = (l + h) / 2;
            let cmp = Self::compare_ip_bytes(&ip, &header.header_sip[m as usize], 16);

            if cmp < 0 {
                h = m - 1;
            } else if cmp > 0 {
                l = m + 1;
            } else {
                sptr = header.header_ptr[if m > 0 { m as usize - 1 } else { 0 }];
                eptr = header.header_ptr[m as usize];
                break;
            }
        }

        if l > h {
            if l == 0 {
                return Ok("Unknown".to_string());
            }
            if (l as usize) < header.header_sip.len() {
                sptr = header.header_ptr[l as usize - 1];
                eptr = header.header_ptr[l as usize];
            } else if h >= 0 && (h as usize) + 1 < header.header_sip.len() {
                sptr = header.header_ptr[h as usize];
                eptr = header.header_ptr[h as usize + 1];
            } else {
                sptr = header.header_ptr[header.header_sip.len() - 1];
                eptr = sptr + self.record_len;
            }
        }

        if sptr == 0 {
            return Ok("Unknown".to_string());
        }

        // Read index block directly from data (no extra allocation)
        let block_len = eptr - sptr;
        let data_offset = self.start_offset + sptr;

        let mut l = 0i32;
        let mut h = (block_len / self.record_len) as i32 - 1;
        let mut data_ptr = 0u32;
        let mut data_len = 0u8;

        while l <= h {
            let m = (l + h) / 2;
            let p = m as usize * self.record_len;
            let slice_offset = data_offset + p;

            let start_ip = u128::from_be_bytes(self.data[slice_offset..slice_offset+16].try_into().unwrap());
            let end_ip = u128::from_be_bytes(self.data[slice_offset+16..slice_offset+32].try_into().unwrap());

            let ip_u128 = u128::from_be_bytes(ip);

            if ip_u128 >= start_ip && ip_u128 <= end_ip {
                data_ptr = LE::read_u32(&self.data[slice_offset+32..slice_offset+36]);
                data_len = self.data[slice_offset+36];
                break;
            } else if ip_u128 < start_ip {
                h = m - 1;
            } else {
                l = m + 1;
            }
        }

        if data_ptr == 0 {
            return Ok("Unknown".to_string());
        }

        self.get_region(data_ptr as usize, data_len as usize)
    }

    /// Compare two IP byte arrays
    fn compare_ip_bytes(ip1: &[u8], ip2: &[u8], len: usize) -> i32 {
        for i in 0..len {
            if ip1[i] < ip2[i] {
                return -1;
            } else if ip1[i] > ip2[i] {
                return 1;
            }
        }
        0
    }

    /// Get region data by pointer and length
    fn get_region(&self, ptr: usize, len: usize) -> Result<String, CzdbError> {
        let offset = self.start_offset + ptr;

        if offset + len > self.data.len() {
            return Err(CzdbError::InvalidFormat);
        }

        let region_bytes = &self.data[offset..offset+len];
        let mut buf = Cursor::new(region_bytes);

        let geo_pos_mix_size = rmp::decode::read_int(&mut buf).unwrap_or(0) as u64;

        let geo_len = ((geo_pos_mix_size >> 24) & 0xFF) as usize;
        let geo_ptr = (geo_pos_mix_size & 0x00FFFFFF) as usize;

        let mut result = String::with_capacity(64);

        if geo_pos_mix_size != 0 {
            if let Some(geo_map_data) = &self.geo_map_data {
                self.append_geo_string(geo_map_data, geo_ptr, geo_len, &mut result)?;
            }
        }

        match rmp::decode::read_str_len(&mut buf) {
            Ok(str_len) => {
                let str_len = str_len as usize;
                let pos = buf.position() as usize;
                if pos + str_len <= region_bytes.len() {
                    let str_bytes = &region_bytes[pos..pos+str_len];
                    result.push_str(&String::from_utf8_lossy(str_bytes));
                }
            }
            Err(_) => {}
        }

        Ok(result)
    }

    fn append_geo_string(&self, geo_map_data: &[u8], ptr: usize, len: usize, result: &mut String) -> Result<(), CzdbError> {
        if ptr + len > geo_map_data.len() {
            return Err(CzdbError::InvalidFormat);
        }

        let data_row = &geo_map_data[ptr..ptr+len];
        let mut buf = Cursor::new(data_row);

        let len = rmp::decode::read_array_len(&mut buf)?;

        let mut first = true;

        for i in 0..len {
            let column_selected = (self.column_selection >> (i + 1) & 1) == 1;

            let str_len = rmp::decode::read_str_len(&mut buf)?;
            let str_len = str_len as usize;
            let pos = buf.position() as usize;

            if pos + str_len > data_row.len() {
                return Err(CzdbError::InvalidFormat);
            }

            if column_selected {
                if !first {
                    result.push('\t');
                }
                let str_bytes = &data_row[pos..pos+str_len];
                result.push_str(&String::from_utf8_lossy(str_bytes));
                first = false;
            }

            buf.set_position((pos + str_len) as u64);
        }

        Ok(())
    }

    /// Get the current search mode
    pub fn search_mode(&self) -> SearchMode {
        self.search_mode
    }
}

#[allow(dead_code)]
struct HyperHeaderBlock {
    version: u32,
    client_id: u32,
    encrypted_block_size: usize,
    decrypted_block: DecryptedBlock,
}

#[allow(dead_code)]
struct DecryptedBlock {
    client_id: u32,
    expiration_date: u32,
    random_size: usize,
}
