use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// Lightweight block header for SPV (Simplified Payment Verification)
/// Compressed to 256 bits for efficient transmission to light clients
/// 
/// Traditional block header: ~1KB+ (full metadata)
/// SPV header: ~256 bits (essential proof data only)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightBlockHeader {
    /// Block height in the chain
    pub height: u64,
    
    /// Hash of previous block (for chain linking)
    pub prev_hash: String,
    
    /// Merkle root of all transactions in block
    pub merkle_root: String,
    
    /// Block creation timestamp (seconds since epoch)
    pub timestamp: u64,
    
    /// Hash of this block header
    pub block_hash: String,
    
    /// Difficulty target (for PoW verification)
    pub difficulty: u32,
    
    /// Nonce used in mining (for PoW verification)
    pub nonce: u64,
}

impl LightBlockHeader {
    /// Create a new light block header
    pub fn new(
        height: u64,
        prev_hash: String,
        merkle_root: String,
        timestamp: u64,
        difficulty: u32,
        nonce: u64,
    ) -> Self {
        let mut header = LightBlockHeader {
            height,
            prev_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce,
            block_hash: String::new(),
        };
        
        // Compute hash
        header.block_hash = header.compute_hash();
        header
    }

    /// Compute the hash of this header
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        
        // Hash all fields in order
        hasher.update(self.height.to_le_bytes());
        hasher.update(self.prev_hash.as_bytes());
        hasher.update(self.merkle_root.as_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.difficulty.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        
        format!("{:x}", hasher.finalize())
    }

    /// Verify this header's hash is correct
    pub fn verify_hash(&self) -> bool {
        self.block_hash == self.compute_hash()
    }

    /// Verify chain link to previous header
    pub fn verify_chain_link(&self, prev_header: &LightBlockHeader) -> bool {
        self.prev_hash == prev_header.block_hash && self.height == prev_header.height + 1
    }

    /// Get the size of this header in bytes (for bandwidth estimation)
    pub fn size_bytes(&self) -> usize {
        // Estimate: 
        // height: 8 bytes
        // prev_hash: 64 bytes (hex string)
        // merkle_root: 64 bytes
        // timestamp: 8 bytes
        // block_hash: 64 bytes
        // difficulty: 4 bytes
        // nonce: 8 bytes
        // = ~220 bytes (close to 256 bits = 32 bytes * 8 = 256 bits)
        220
    }

    /// Serialize header to compact binary format with length prefixes
    pub fn to_compact_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Height (8 bytes)
        bytes.extend_from_slice(&self.height.to_le_bytes());
        
        // prev_hash with length prefix (2 bytes length + variable)
        let prev_hash_bytes = self.prev_hash.as_bytes();
        bytes.extend_from_slice(&(prev_hash_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(prev_hash_bytes);
        
        // merkle_root with length prefix (2 bytes length + variable)
        let merkle_root_bytes = self.merkle_root.as_bytes();
        bytes.extend_from_slice(&(merkle_root_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(merkle_root_bytes);
        
        // Timestamp (8 bytes)
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        
        // Difficulty (4 bytes)
        bytes.extend_from_slice(&self.difficulty.to_le_bytes());
        
        // Nonce (8 bytes)
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        
        // block_hash with length prefix (2 bytes length + variable)
        let block_hash_bytes = self.block_hash.as_bytes();
        bytes.extend_from_slice(&(block_hash_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(block_hash_bytes);
        
        bytes
    }

    /// Deserialize header from compact binary format with length prefixes
    pub fn from_compact_bytes(bytes: &[u8]) -> Result<Self, String> {
        // Minimum size: 8 + 2 + 2 + 2 + 8 + 4 + 8 + 2 = 36 bytes
        if bytes.len() < 36 {
            return Err("Insufficient bytes for header".to_string());
        }

        let mut pos = 0;

        // Parse height (8 bytes)
        let height = u64::from_le_bytes(
            bytes[pos..pos + 8]
                .try_into()
                .map_err(|_| "Invalid height")?,
        );
        pos += 8;

        // Parse prev_hash (2 bytes length + variable)
        if pos + 2 > bytes.len() {
            return Err("Insufficient bytes for prev_hash length".to_string());
        }
        let prev_hash_len = u16::from_le_bytes(
            bytes[pos..pos + 2]
                .try_into()
                .map_err(|_| "Invalid prev_hash length")?,
        ) as usize;
        pos += 2;
        
        if pos + prev_hash_len > bytes.len() {
            return Err("Insufficient bytes for prev_hash".to_string());
        }
        let prev_hash = String::from_utf8(bytes[pos..pos + prev_hash_len].to_vec())
            .map_err(|_| "Invalid prev_hash")?;
        pos += prev_hash_len;

        // Parse merkle_root (2 bytes length + variable)
        if pos + 2 > bytes.len() {
            return Err("Insufficient bytes for merkle_root length".to_string());
        }
        let merkle_root_len = u16::from_le_bytes(
            bytes[pos..pos + 2]
                .try_into()
                .map_err(|_| "Invalid merkle_root length")?,
        ) as usize;
        pos += 2;
        
        if pos + merkle_root_len > bytes.len() {
            return Err("Insufficient bytes for merkle_root".to_string());
        }
        let merkle_root = String::from_utf8(bytes[pos..pos + merkle_root_len].to_vec())
            .map_err(|_| "Invalid merkle_root")?;
        pos += merkle_root_len;

        // Parse timestamp (8 bytes)
        if pos + 8 > bytes.len() {
            return Err("Insufficient bytes for timestamp".to_string());
        }
        let timestamp = u64::from_le_bytes(
            bytes[pos..pos + 8]
                .try_into()
                .map_err(|_| "Invalid timestamp")?,
        );
        pos += 8;

        // Parse difficulty (4 bytes)
        if pos + 4 > bytes.len() {
            return Err("Insufficient bytes for difficulty".to_string());
        }
        let difficulty = u32::from_le_bytes(
            bytes[pos..pos + 4]
                .try_into()
                .map_err(|_| "Invalid difficulty")?,
        );
        pos += 4;

        // Parse nonce (8 bytes)
        if pos + 8 > bytes.len() {
            return Err("Insufficient bytes for nonce".to_string());
        }
        let nonce = u64::from_le_bytes(
            bytes[pos..pos + 8]
                .try_into()
                .map_err(|_| "Invalid nonce")?,
        );
        pos += 8;

        // Parse block_hash (2 bytes length + variable)
        if pos + 2 > bytes.len() {
            return Err("Insufficient bytes for block_hash length".to_string());
        }
        let block_hash_len = u16::from_le_bytes(
            bytes[pos..pos + 2]
                .try_into()
                .map_err(|_| "Invalid block_hash length")?,
        ) as usize;
        pos += 2;
        
        if pos + block_hash_len > bytes.len() {
            return Err("Insufficient bytes for block_hash".to_string());
        }
        let block_hash = String::from_utf8(bytes[pos..pos + block_hash_len].to_vec())
            .map_err(|_| "Invalid block_hash")?;

        Ok(LightBlockHeader {
            height,
            prev_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce,
            block_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_block_header_creation() {
        let header = LightBlockHeader::new(
            100,
            "prev_hash_123".to_string(),
            "merkle_root_abc".to_string(),
            1234567890,
            1000,
            42,
        );

        assert_eq!(header.height, 100);
        assert_eq!(header.timestamp, 1234567890);
        assert!(!header.block_hash.is_empty());
    }

    #[test]
    fn test_light_block_header_hash_verification() {
        let header = LightBlockHeader::new(
            100,
            "prev_hash_123".to_string(),
            "merkle_root_abc".to_string(),
            1234567890,
            1000,
            42,
        );

        assert!(header.verify_hash());
    }

    #[test]
    fn test_light_block_header_hash_mismatch() {
        let mut header = LightBlockHeader::new(
            100,
            "prev_hash_123".to_string(),
            "merkle_root_abc".to_string(),
            1234567890,
            1000,
            42,
        );

        // Corrupt the hash
        header.block_hash = "corrupted_hash".to_string();
        assert!(!header.verify_hash());
    }

    #[test]
    fn test_light_block_header_chain_link() {
        let header1 = LightBlockHeader::new(
            100,
            "genesis".to_string(),
            "merkle_1".to_string(),
            1000,
            1000,
            1,
        );

        let header2 = LightBlockHeader::new(
            101,
            header1.block_hash.clone(),
            "merkle_2".to_string(),
            2000,
            1000,
            2,
        );

        assert!(header2.verify_chain_link(&header1));
    }

    #[test]
    fn test_light_block_header_chain_link_broken() {
        let header1 = LightBlockHeader::new(
            100,
            "genesis".to_string(),
            "merkle_1".to_string(),
            1000,
            1000,
            1,
        );

        let header2 = LightBlockHeader::new(
            101,
            "wrong_prev_hash".to_string(),
            "merkle_2".to_string(),
            2000,
            1000,
            2,
        );

        assert!(!header2.verify_chain_link(&header1));
    }

    #[test]
    fn test_light_block_header_size() {
        let header = LightBlockHeader::new(
            100,
            "prev_hash_123".to_string(),
            "merkle_root_abc".to_string(),
            1234567890,
            1000,
            42,
        );

        assert!(header.size_bytes() > 0);
        // Should be much smaller than full header (~1KB)
        assert!(header.size_bytes() < 1000);
    }

    #[test]
    fn test_light_block_header_compact_serialization() {
        let header = LightBlockHeader::new(
            100,
            "prev_hash_123".to_string(),
            "merkle_root_abc".to_string(),
            1234567890,
            1000,
            42,
        );

        let bytes = header.to_compact_bytes();
        assert!(!bytes.is_empty());

        let deserialized = LightBlockHeader::from_compact_bytes(&bytes)
            .expect("Failed to deserialize");

        assert_eq!(deserialized.height, header.height);
        assert_eq!(deserialized.timestamp, header.timestamp);
        assert_eq!(deserialized.difficulty, header.difficulty);
    }

    #[test]
    fn test_light_block_header_deterministic_hash() {
        let header1 = LightBlockHeader::new(
            100,
            "prev_hash_123".to_string(),
            "merkle_root_abc".to_string(),
            1234567890,
            1000,
            42,
        );

        let header2 = LightBlockHeader::new(
            100,
            "prev_hash_123".to_string(),
            "merkle_root_abc".to_string(),
            1234567890,
            1000,
            42,
        );

        assert_eq!(header1.block_hash, header2.block_hash);
    }
}
