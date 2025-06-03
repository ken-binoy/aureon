use sha2::{Sha256, Digest};

pub fn derive_address_from_seed(seed: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..20]) // Truncated hash
}