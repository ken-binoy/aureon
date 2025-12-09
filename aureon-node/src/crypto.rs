/// Cryptographic utilities for Ed25519 signature generation and verification
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, SignatureError, Signer, Verifier};
use rand::rngs::StdRng;
use rand::SeedableRng;
use sha2::{Sha256, Digest};
use hex::encode as hex_encode;

/// Generate a new Ed25519 keypair
/// Returns (secret_key_hex, public_key_hex)
pub fn generate_keypair() -> (String, String) {
    let mut rng = StdRng::from_entropy();
    let secret_bytes: [u8; 32] = {
        let mut bytes = [0u8; 32];
        use rand::RngCore;
        rng.fill_bytes(&mut bytes);
        bytes
    };
    
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();
    
    let public_bytes = verifying_key.to_bytes();
    
    (hex_encode(secret_bytes), hex_encode(public_bytes))
}

/// Sign a message with an Ed25519 secret key
pub fn sign_message(message: &[u8], secret_key_hex: &str) -> Result<String, String> {
    // Decode the hex secret key
    let secret_bytes = hex::decode(secret_key_hex)
        .map_err(|e| format!("Invalid secret key format: {}", e))?;
    
    if secret_bytes.len() != 32 {
        return Err("Secret key must be 32 bytes".to_string());
    }
    
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&secret_bytes);
    
    let signing_key = SigningKey::from_bytes(&key_array);
    let signature = signing_key.sign(message);
    
    Ok(hex_encode(signature.to_bytes()))
}

/// Verify a signature with an Ed25519 public key
pub fn verify_signature(message: &[u8], signature_hex: &str, public_key_hex: &str) -> Result<bool, String> {
    // Decode signature
    let signature_bytes = hex::decode(signature_hex)
        .map_err(|e| format!("Invalid signature format: {}", e))?;
    
    if signature_bytes.len() != 64 {
        return Err("Signature must be 64 bytes".to_string());
    }
    
    // Decode public key
    let public_bytes = hex::decode(public_key_hex)
        .map_err(|e| format!("Invalid public key format: {}", e))?;
    
    if public_bytes.len() != 32 {
        return Err("Public key must be 32 bytes".to_string());
    }
    
    // Create signature and public key objects
    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&signature_bytes);
    let signature = Signature::from_bytes(&sig_array);
    
    let mut pub_array = [0u8; 32];
    pub_array.copy_from_slice(&public_bytes);
    let verifying_key = VerifyingKey::from_bytes(&pub_array)
        .map_err(|e: SignatureError| format!("Invalid public key: {}", e))?;
    
    // Verify the signature
    match verifying_key.verify(message, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Compute the transaction hash (used for signing)
pub fn compute_transaction_hash(tx_data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tx_data);
    hex_encode(hasher.finalize())
}

/// Compute public key address from public key (Ethereum-style, first 20 bytes of keccak256)
pub fn public_key_to_address(public_key_hex: &str) -> Result<String, String> {
    let public_bytes = hex::decode(public_key_hex)
        .map_err(|e| format!("Invalid public key format: {}", e))?;
    
    if public_bytes.len() != 32 {
        return Err("Public key must be 32 bytes".to_string());
    }
    
    // Hash the public key with SHA256 and take first 20 bytes (40 hex chars)
    let mut hasher = Sha256::new();
    hasher.update(&public_bytes);
    let hash = hasher.finalize();
    let address = hex_encode(&hash[0..20]);
    
    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (secret, public) = generate_keypair();
        assert_eq!(secret.len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(public.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_sign_and_verify() {
        let message = b"hello world";
        let (secret, public) = generate_keypair();
        
        // Sign the message
        let signature = sign_message(message, &secret).expect("Failed to sign");
        assert_eq!(signature.len(), 128); // 64 bytes = 128 hex chars
        
        // Verify the signature
        let is_valid = verify_signature(message, &signature, &public).expect("Failed to verify");
        assert!(is_valid);
        
        // Verify with wrong message should fail
        let is_valid_wrong = verify_signature(b"different message", &signature, &public).expect("Failed to verify");
        assert!(!is_valid_wrong);
    }

    #[test]
    fn test_public_key_to_address() {
        let (_, public) = generate_keypair();
        let address = public_key_to_address(&public).expect("Failed to compute address");
        assert_eq!(address.len(), 40); // 20 bytes = 40 hex chars
    }

    #[test]
    fn test_compute_transaction_hash() {
        let data = b"transaction data";
        let hash = compute_transaction_hash(data);
        assert_eq!(hash.len(), 64); // SHA256 = 256 bits = 64 hex chars
    }

    #[test]
    fn test_invalid_secret_key_format() {
        let result = sign_message(b"message", "not_hex");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_signature_format() {
        let (_, public) = generate_keypair();
        let result = verify_signature(b"message", "not_hex", &public);
        assert!(result.is_err());
    }
}
