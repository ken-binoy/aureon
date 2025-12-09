/// Utility module for key generation and transaction signing
/// Use this to generate Ed25519 keypairs and sign transactions
use crate::crypto;
use serde_json::json;

/// Generate a new Ed25519 keypair and return as JSON
pub fn generate_keypair_json() -> serde_json::Value {
    let (secret_key, public_key) = crypto::generate_keypair();
    
    json!({
        "secret_key": secret_key,
        "public_key": public_key,
        "message": "Store secret_key safely. Use it to sign transactions.",
        "usage": "POST to /submit-signed-tx with public_key and signature"
    })
}

/// Sign a transaction given secret key and transaction parameters
pub fn sign_transaction(
    secret_key: &str,
    from: &str,
    to: &str,
    amount: u64,
    nonce: u64,
) -> Result<serde_json::Value, String> {
    // Create a deterministic message from transaction data
    let tx_data = format!("{}:{}:{}:{}", from, to, amount, nonce);
    
    let signature = crypto::sign_message(tx_data.as_bytes(), secret_key)?;
    
    // Extract public key from secret key (derive it)
    // For simplicity, we'll return an error message asking for public key
    // In production, you'd have a way to derive it
    
    Ok(json!({
        "signature": signature,
        "tx_data": tx_data,
        "message": "Use signature and public_key with /submit-signed-tx endpoint"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair_json() {
        let result = generate_keypair_json();
        assert!(result["secret_key"].is_string());
        assert!(result["public_key"].is_string());
        assert_eq!(result["secret_key"].as_str().unwrap().len(), 64);
        assert_eq!(result["public_key"].as_str().unwrap().len(), 64);
    }

    #[test]
    fn test_sign_transaction() {
        let (secret, _public) = crypto::generate_keypair();
        let result = sign_transaction(&secret, "Alice", "Bob", 100, 0);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json["signature"].is_string());
        assert!(json["tx_data"].is_string());
    }
}
