use std::collections::HashMap;
use crate::light_block_header::LightBlockHeader;
use crate::merkle_tree::MerkleInclusionProof;

/// Result of SPV verification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationResult {
    /// Transaction is valid according to SPV rules
    Valid,
    /// Transaction is invalid
    Invalid,
    /// Not enough confirmations yet
    InsufficientConfirmations,
    /// Proof is malformed or incomplete
    MalformedProof,
}

/// SPV (Simplified Payment Verification) Client
/// Verifies transactions without storing full blockchain
/// Requires only headers and merkle inclusion proofs
pub struct SpvClient {
    /// Chain of block headers
    headers: Vec<LightBlockHeader>,
    
    /// Mapping of block hash to header for quick lookup
    header_map: HashMap<String, usize>,
    
    /// Number of confirmations required for validity
    confirmations_required: u64,
}

impl SpvClient {
    /// Create a new SPV client
    pub fn new(confirmations_required: u64) -> Self {
        SpvClient {
            headers: Vec::new(),
            header_map: HashMap::new(),
            confirmations_required,
        }
    }

    /// Add a block header to the chain
    /// Returns true if header was added, false if it conflicts with chain
    pub fn add_header(&mut self, header: LightBlockHeader) -> bool {
        // Verify header hash is correct
        if !header.verify_hash() {
            return false;
        }

        // If this is not the genesis header, verify chain link
        if !self.headers.is_empty() {
            let prev_header = &self.headers[self.headers.len() - 1];
            if !header.verify_chain_link(prev_header) {
                return false;
            }
        }

        // Add to header chain
        let index = self.headers.len();
        self.header_map.insert(header.block_hash.clone(), index);
        self.headers.push(header);

        true
    }

    /// Add multiple headers at once (batch operation)
    pub fn add_headers(&mut self, headers: Vec<LightBlockHeader>) -> usize {
        let mut added = 0;
        for header in headers {
            if self.add_header(header) {
                added += 1;
            }
        }
        added
    }

    /// Get a header by block hash
    pub fn get_header(&self, block_hash: &str) -> Option<&LightBlockHeader> {
        self.header_map
            .get(block_hash)
            .and_then(|&idx| self.headers.get(idx))
    }

    /// Get the latest header
    pub fn get_latest_header(&self) -> Option<&LightBlockHeader> {
        self.headers.last()
    }

    /// Get the height of the latest header
    pub fn chain_height(&self) -> u64 {
        self.headers.last().map(|h| h.height).unwrap_or(0)
    }

    /// Verify a transaction using SPV
    /// Requires:
    /// 1. Block hash where transaction is included
    /// 2. Transaction hash
    /// 3. Merkle inclusion proof
    /// 4. Sufficient confirmations
    pub fn verify_transaction(
        &self,
        block_hash: &str,
        tx_hash: &str,
        proof: &MerkleInclusionProof,
    ) -> VerificationResult {
        // Check if we have the block header
        let block_header = match self.get_header(block_hash) {
            Some(h) => h,
            None => return VerificationResult::Invalid,
        };

        // Verify the merkle root in proof matches block header
        if proof.merkle_root != block_header.merkle_root {
            return VerificationResult::Invalid;
        }

        // Verify the transaction hash
        if proof.tx_hash != tx_hash {
            return VerificationResult::Invalid;
        }

        // Verify the merkle inclusion proof
        if !proof.verify() {
            return VerificationResult::MalformedProof;
        }

        // Check confirmations
        let confirmations = self.get_confirmations(block_hash);
        if confirmations < self.confirmations_required {
            return VerificationResult::InsufficientConfirmations;
        }

        VerificationResult::Valid
    }

    /// Get number of confirmations for a block
    /// Confirmations = height difference between latest and block + 1
    pub fn get_confirmations(&self, block_hash: &str) -> u64 {
        match self.get_header(block_hash) {
            Some(header) => {
                let latest_height = self.chain_height();
                if header.height <= latest_height {
                    latest_height - header.height + 1
                } else {
                    0
                }
            }
            None => 0,
        }
    }

    /// Estimate transaction safety based on confirmations
    pub fn is_transaction_safe(&self, block_hash: &str) -> bool {
        self.get_confirmations(block_hash) >= self.confirmations_required
    }

    /// Get total number of headers synchronized
    pub fn header_count(&self) -> usize {
        self.headers.len()
    }

    /// Estimate blockchain size if full node was used
    /// Each header ~220 bytes, extrapolate to full blocks ~1KB
    pub fn estimated_full_node_size(&self) -> u64 {
        let header_count = self.headers.len() as u64;
        header_count * 1024  // ~1KB per block average
    }

    /// Actual storage used by SPV client (just headers)
    pub fn spv_storage_used(&self) -> u64 {
        self.headers.len() as u64 * 220  // ~220 bytes per header
    }

    /// Calculate space savings vs full node
    pub fn space_savings_percentage(&self) -> f64 {
        if self.header_count() == 0 {
            return 0.0;
        }
        let full = self.estimated_full_node_size();
        let spv = self.spv_storage_used();
        ((full - spv) as f64 / full as f64) * 100.0
    }

    /// Verify entire header chain from genesis
    pub fn verify_chain(&self) -> bool {
        if self.headers.is_empty() {
            return true;  // Empty chain is valid
        }

        // Check each header
        for (i, header) in self.headers.iter().enumerate() {
            // Verify hash
            if !header.verify_hash() {
                return false;
            }

            // Verify chain link (except genesis)
            if i > 0 {
                if !header.verify_chain_link(&self.headers[i - 1]) {
                    return false;
                }
            }
        }

        true
    }

    /// Get headers in a range
    pub fn get_headers_in_range(&self, from_height: u64, to_height: u64) -> Vec<&LightBlockHeader> {
        self.headers
            .iter()
            .filter(|h| h.height >= from_height && h.height <= to_height)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_header(height: u64, prev_hash: String) -> LightBlockHeader {
        LightBlockHeader::new(
            height,
            prev_hash,
            format!("merkle_{}", height),
            1000 + height,
            1000,
            height as u64,
        )
    }

    #[test]
    fn test_spv_client_creation() {
        let client = SpvClient::new(6);
        assert_eq!(client.header_count(), 0);
        assert_eq!(client.chain_height(), 0);
    }

    #[test]
    fn test_spv_add_genesis_header() {
        let mut client = SpvClient::new(1);
        let genesis = create_test_header(0, "0x00".to_string());

        assert!(client.add_header(genesis));
        assert_eq!(client.header_count(), 1);
        assert_eq!(client.chain_height(), 0);
    }

    #[test]
    fn test_spv_add_header_chain() {
        let mut client = SpvClient::new(1);

        let header0 = create_test_header(0, "0x00".to_string());
        let hash0 = header0.block_hash.clone();

        assert!(client.add_header(header0));

        let header1 = create_test_header(1, hash0);
        let hash1 = header1.block_hash.clone();

        assert!(client.add_header(header1.clone()));
        assert_eq!(client.header_count(), 2);
        assert_eq!(client.chain_height(), 1);

        // Add another
        let header2 = create_test_header(2, hash1);
        assert!(client.add_header(header2));
        assert_eq!(client.header_count(), 3);
        assert_eq!(client.chain_height(), 2);
    }

    #[test]
    fn test_spv_add_headers_batch() {
        let mut client = SpvClient::new(1);

        let header0 = create_test_header(0, "0x00".to_string());
        let hash0 = header0.block_hash.clone();

        client.add_header(header0);

        let header1 = create_test_header(1, hash0.clone());
        let hash1 = header1.block_hash.clone();

        let header2 = create_test_header(2, hash1);

        let added = client.add_headers(vec![header1, header2]);
        assert_eq!(added, 2);
        assert_eq!(client.header_count(), 3);
    }

    #[test]
    fn test_spv_get_header() {
        let mut client = SpvClient::new(1);
        let header = create_test_header(0, "0x00".to_string());
        let hash = header.block_hash.clone();

        client.add_header(header.clone());
        assert_eq!(client.get_header(&hash), Some(&header));
    }

    #[test]
    fn test_spv_get_latest_header() {
        let mut client = SpvClient::new(1);

        let header0 = create_test_header(0, "0x00".to_string());
        let hash0 = header0.block_hash.clone();
        client.add_header(header0);

        let header1 = create_test_header(1, hash0);
        let header1_copy = header1.clone();
        client.add_header(header1);

        assert_eq!(client.get_latest_header(), Some(&header1_copy));
    }

    #[test]
    fn test_spv_confirmations() {
        let mut client = SpvClient::new(1);

        let header0 = create_test_header(0, "0x00".to_string());
        let hash0 = header0.block_hash.clone();
        client.add_header(header0);

        let header1 = create_test_header(1, hash0.clone());
        client.add_header(header1);

        // Block 0 should have 2 confirmations (at height 1)
        assert_eq!(client.get_confirmations(&hash0), 2);
    }

    #[test]
    fn test_spv_is_transaction_safe() {
        let mut client = SpvClient::new(3);

        let header0 = create_test_header(0, "0x00".to_string());
        let hash0 = header0.block_hash.clone();
        client.add_header(header0);

        // Not enough confirmations yet
        assert!(!client.is_transaction_safe(&hash0));

        // Add more headers
        let mut prev_hash = hash0.clone();
        for i in 1..=3 {
            let header = create_test_header(i, prev_hash.clone());
            prev_hash = header.block_hash.clone();
            client.add_header(header);
        }

        // Now it should be safe (4 confirmations, need 3)
        assert!(client.is_transaction_safe(&hash0));
    }

    #[test]
    fn test_spv_verify_chain() {
        let mut client = SpvClient::new(1);

        let header0 = create_test_header(0, "0x00".to_string());
        let hash0 = header0.block_hash.clone();
        client.add_header(header0);

        let header1 = create_test_header(1, hash0);
        client.add_header(header1);

        assert!(client.verify_chain());
    }

    #[test]
    fn test_spv_storage_efficiency() {
        let mut client = SpvClient::new(1);

        // Add 100 headers
        let mut prev_hash = "0x00".to_string();
        for i in 0..100 {
            let header = create_test_header(i, prev_hash.clone());
            prev_hash = header.block_hash.clone();
            client.add_header(header);
        }

        let spv_size = client.spv_storage_used();
        let full_size = client.estimated_full_node_size();
        let savings = client.space_savings_percentage();

        assert!(spv_size < full_size);
        assert!(savings > 75.0);  // Should save over 75%
    }

    #[test]
    fn test_spv_get_headers_in_range() {
        let mut client = SpvClient::new(1);

        let header0 = create_test_header(0, "0x00".to_string());
        let hash0 = header0.block_hash.clone();
        client.add_header(header0);

        let header1 = create_test_header(1, hash0.clone());
        let hash1 = header1.block_hash.clone();
        client.add_header(header1.clone());

        let header2 = create_test_header(2, hash1);
        client.add_header(header2);

        let range = client.get_headers_in_range(0, 2);
        assert_eq!(range.len(), 3);

        let range = client.get_headers_in_range(1, 1);
        assert_eq!(range.len(), 1);
    }
}
