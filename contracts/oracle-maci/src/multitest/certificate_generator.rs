use base64;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use serde_json;
use sha2::{Digest, Sha256};

// Private key (hex format)
const PRIVATE_KEY_HEX: &str = "";

/// Generate certificate for given user address and amount
pub fn generate_certificate(
    contract_address: &str,
    user_address: &str,
    amount: u128,
    ecosystem: &str,
) -> String {
    // Create payload
    let payload = serde_json::json!({
        "contract_address": contract_address,
        "address": user_address,
        "amount": amount.to_string(),
        "ecosystem": ecosystem,
    });

    let msg = payload.to_string().into_bytes();
    let hash = Sha256::digest(&msg);

    // Parse private key
    let private_key_bytes = hex::decode(PRIVATE_KEY_HEX).expect("Invalid private key hex");
    let secret_key = SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");

    // Create secp256k1 context
    let secp = Secp256k1::new();

    // Create message object
    let message = Message::from_slice(&hash).expect("32 bytes");

    // Sign
    let signature = secp.sign_ecdsa(&message, &secret_key);

    // Serialize signature and convert to base64
    let signature_bytes = signature.serialize_compact();
    base64::encode(&signature_bytes)
}

/// Verify if public key matches private key
pub fn verify_keypair() -> bool {
    let private_key_bytes = hex::decode(PRIVATE_KEY_HEX).expect("Invalid private key hex");
    let secret_key = SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");

    let secp = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    // Expected public key (base64)
    let expected_pubkey_b64 = "AurWMPImmkDdmse2sMILI9g0fJlOfcnxjJ1QWyFRBdXr";
    let expected_pubkey_bytes = base64::decode(expected_pubkey_b64).expect("Invalid base64");

    // Compare public keys
    public_key.serialize().to_vec() == expected_pubkey_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_match() {
        assert!(verify_keypair(), "Public key and private key do not match");
    }

    #[test]
    fn test_certificate_generation() {
        let certificate = generate_certificate("contract0", "0", 100000000u128, "cosmoshub");

        // Ensure certificate is not empty
        assert!(!certificate.is_empty());

        // Ensure certificate is valid base64
        assert!(base64::decode(&certificate).is_ok());

        println!("Generated certificate for user 0: {}", certificate);
    }

    #[test]
    fn test_different_users() {
        let cert0 = generate_certificate("contract0", "0", 100000000u128, "cosmoshub");
        let cert1 = generate_certificate("contract0", "1", 80000000u128, "cosmoshub");

        // Different users should have different certificates
        assert_ne!(cert0, cert1);

        println!("User 0 certificate: {}", cert0);
        println!("User 1 certificate: {}", cert1);
    }
}
