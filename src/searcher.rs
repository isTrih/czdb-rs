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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IpType {
    Ipv4,
    Ipv6,
}

pub struct DbSearcher {
    data: Vec<u8>,
    start_offset: usize,
    index_start_offset: usize,
    ip_type: IpType,
    geo_map_data: Option<Vec<u8>>,
    column_selection: u32,
    
    // Memory mode indices
    index_v4_keys: Vec<u32>,
    index_v6_keys: Vec<u128>,
}

impl DbSearcher {
    pub fn new(data: Vec<u8>, key: &str) -> Result<Self, CzdbError> {
        let (_header_block, offset) = Self::parse_header(&data, key)?;
        
        // Read Super Header (17 bytes) at offset
        if data.len() < offset + 17 {
            return Err(CzdbError::InvalidFormat);
        }
        let super_header = &data[offset..offset + 17];
        
        let ip_type = if super_header[0] & 1 == 0 { IpType::Ipv4 } else { IpType::Ipv6 };
        let header_block_size = LE::read_u32(&super_header[5..9]) as usize;
        let start_index_ptr = header_block_size;
        let end_index_ptr = LE::read_u32(&super_header[13..17]) as usize;
        
        let mut searcher = DbSearcher {
            data,
            start_offset: offset,
            index_start_offset: offset + start_index_ptr,
            ip_type,
            geo_map_data: None,
            column_selection: 0,
            index_v4_keys: Vec::new(),
            index_v6_keys: Vec::new(),
        };

        searcher.load_geo_mapping(key, end_index_ptr)?;
        searcher.load_index(start_index_ptr, end_index_ptr)?;

        Ok(searcher)
    }

    fn load_index(&mut self, start_ptr: usize, end_ptr: usize) -> Result<(), CzdbError> {
        let start_offset = self.start_offset + start_ptr;
        let end_offset = self.start_offset + end_ptr;
        
        if end_offset > self.data.len() {
            return Err(CzdbError::InvalidFormat);
        }

        match self.ip_type {
            IpType::Ipv4 => {
                let record_len = 13; // 4 + 4 + 4 + 1
                let count = (end_ptr - start_ptr) / record_len + 1;
                self.index_v4_keys.reserve(count);
                
                let mut ptr = start_offset;
                while ptr + record_len <= self.data.len() && ptr <= end_offset {
                    let start_ip = u32::from_be_bytes(self.data[ptr..ptr+4].try_into().unwrap());
                    self.index_v4_keys.push(start_ip);
                    ptr += record_len;
                }
            },
            IpType::Ipv6 => {
                let record_len = 37; // 16 + 16 + 4 + 1
                let count = (end_ptr - start_ptr) / record_len + 1;
                self.index_v6_keys.reserve(count);
                
                let mut ptr = start_offset;
                while ptr + record_len <= self.data.len() && ptr <= end_offset {
                    let start_ip = u128::from_be_bytes(self.data[ptr..ptr+16].try_into().unwrap());
                    self.index_v6_keys.push(start_ip);
                    ptr += record_len;
                }
            }
        }
        Ok(())
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
        
        // Decrypted block structure:
        // Bytes 0-4: ClientID (high 12 bits) | ExpirationDate (low 20 bits)
        // Bytes 4-8: Random Size
        if decrypted_bytes.len() < 8 {
            return Err(CzdbError::InvalidFormat);
        }
        
        let first_u32 = LE::read_u32(&decrypted_bytes[0..4]);
        let decrypted_client_id = first_u32 >> 20;
        let expiration_date = first_u32 & 0xFFFFF;
        let random_size = LE::read_u32(&decrypted_bytes[4..8]) as usize;
        
        if decrypted_client_id != client_id {
            return Err(CzdbError::ClientIdMismatch);
        }
        
        // Check expiration (optional, maybe skip for now or implement)
        // let current_date = ...
        
        let header_block = HyperHeaderBlock {
            version,
            client_id,
            encrypted_block_size,
            decrypted_block: DecryptedBlock {
                client_id: decrypted_client_id,
                expiration_date,
                random_size,
            },
        };
        
        let offset = 12 + encrypted_block_size + random_size;
        Ok((header_block, offset))
    }

    fn load_geo_mapping(&mut self, key: &str, end_index_ptr: usize) -> Result<(), CzdbError> {
        let ip_bytes_len = if self.ip_type == IpType::Ipv4 { 4 } else { 16 };
        let column_selection_ptr = self.start_offset + end_index_ptr + ip_bytes_len * 2 + 5;
        
        if self.data.len() < column_selection_ptr + 4 {
            return Ok(());
        }
        
        self.column_selection = LE::read_u32(&self.data[column_selection_ptr..column_selection_ptr + 4]);
        
        if self.column_selection == 0 {
            return Ok(());
        }
        
        let geo_map_ptr = column_selection_ptr + 4;
        if self.data.len() < geo_map_ptr + 4 {
            return Err(CzdbError::InvalidFormat);
        }
        
        let geo_map_size = LE::read_u32(&self.data[geo_map_ptr..geo_map_ptr + 4]) as usize;
        let geo_map_data_ptr = geo_map_ptr + 4;
        
        if self.data.len() < geo_map_data_ptr + geo_map_size {
            return Err(CzdbError::InvalidFormat);
        }
        
        let mut geo_map_data = self.data[geo_map_data_ptr..geo_map_data_ptr + geo_map_size].to_vec();
        decrypt_xor(key, &mut geo_map_data)?;
        
        self.geo_map_data = Some(geo_map_data);
        
        Ok(())
    }

    pub fn search(&self, ip: &str) -> Result<String, CzdbError> {
        let ip_addr = IpAddr::from_str(ip)?;
        
        match (self.ip_type, ip_addr) {
            (IpType::Ipv4, IpAddr::V4(addr)) => self.search_ipv4(addr.octets()),
            (IpType::Ipv6, IpAddr::V6(addr)) => self.search_ipv6(addr.octets()),
            _ => Err(CzdbError::InvalidIpType),
        }
    }

    fn search_ipv4(&self, ip: [u8; 4]) -> Result<String, CzdbError> {
        let ip_u32 = u32::from_be_bytes(ip);
        
        let idx = match self.index_v4_keys.binary_search(&ip_u32) {
            Ok(i) => i,
            Err(i) => if i > 0 { i - 1 } else { return Ok("Unknown".to_string()) },
        };
        
        // Read full record from data
        let record_len = 13;
        let offset = self.index_start_offset + idx * record_len;
        
        if offset + record_len > self.data.len() {
            return Ok("Unknown".to_string());
        }
        
        // We already know start_ip <= ip_u32 (from binary search logic)
        // Just check end_ip
        let end_ip = u32::from_be_bytes(self.data[offset+4..offset+8].try_into().unwrap());
        
        if ip_u32 <= end_ip {
            let data_ptr = LE::read_u32(&self.data[offset+8..offset+12]);
            let data_len = self.data[offset+12];
            return self.get_region(data_ptr as usize, data_len as usize);
        }
        
        Ok("Unknown".to_string())
    }

    fn search_ipv6(&self, ip: [u8; 16]) -> Result<String, CzdbError> {
        let ip_u128 = u128::from_be_bytes(ip);
        
        let idx = match self.index_v6_keys.binary_search(&ip_u128) {
            Ok(i) => i,
            Err(i) => if i > 0 { i - 1 } else { return Ok("Unknown".to_string()) },
        };
        
        let record_len = 37;
        let offset = self.index_start_offset + idx * record_len;
        
        if offset + record_len > self.data.len() {
            return Ok("Unknown".to_string());
        }
        
        let end_ip = u128::from_be_bytes(self.data[offset+16..offset+32].try_into().unwrap());
        
        if ip_u128 <= end_ip {
            let data_ptr = LE::read_u32(&self.data[offset+32..offset+36]);
            let data_len = self.data[offset+36];
            return self.get_region(data_ptr as usize, data_len as usize);
        }
        
        Ok("Unknown".to_string())
    }



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
        
        // Read otherDataObj
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
        
        // Unpack array
        let len = rmp::decode::read_array_len(&mut buf)?;
        
        let mut first = true;
        
        for i in 0..len {
            let column_selected = (self.column_selection >> (i + 1) & 1) == 1;
            
            // Read string
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
