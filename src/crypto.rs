use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce as GcmNonce,
};
use aes_gcm_siv::{Aes256GcmSiv, Nonce as GcmSivNonce};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use pbkdf2::pbkdf2_hmac_array;
use rand::{rngs::OsRng, RngCore};
use sha2::Sha256;
use thiserror::Error;
use zeroize::Zeroize;

const PREFIX_V1: &str = "hashit:v1:";
const PREFIX_V2: &str = "hashit:v2:";
const VERSION_V1: u8 = 1;
const VERSION_V2: u8 = 2;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const TAG_LEN: usize = 16;
const ITERATIONS: u32 = 210_000;
const V2_KDF_SALT: &[u8] = b"hashit:v2:deterministic-key";
const V2_NONCE: [u8; NONCE_LEN] = [0_u8; NONCE_LEN];

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Metin boş olamaz.")]
    EmptyInput,
    #[error("Master key boş olamaz.")]
    EmptyMasterKey,
    #[error("Şifreli veri bozuk, eksik veya Hashit formatında değil.")]
    InvalidPayload,
    #[error("Güvenli rastgele veri üretilemedi.")]
    RandomGenerationFailed,
    #[error("Master key'den anahtar türetilemedi.")]
    KeyDerivationFailed,
    #[error("Şifreleme başarısız oldu.")]
    EncryptionFailed,
    #[error("Şifre çözme başarısız. Master key yanlış olabilir veya veri bozulmuş olabilir.")]
    DecryptionFailed,
}

pub struct EncryptedPayload;

impl EncryptedPayload {
    pub fn encrypt(plaintext: &str, master_key: &str) -> Result<String, CryptoError> {
        validate_encrypt_inputs(plaintext, master_key)?;
        Self::encrypt_v2(plaintext, master_key)
    }

    pub fn decrypt(encoded: &str, master_key: &str) -> Result<String, CryptoError> {
        let encoded = encoded.trim();

        if encoded.is_empty() {
            return Err(CryptoError::EmptyInput);
        }

        if master_key.is_empty() {
            return Err(CryptoError::EmptyMasterKey);
        }

        if encoded.starts_with(PREFIX_V2) {
            return Self::decrypt_v2(encoded, master_key);
        }

        if encoded.starts_with(PREFIX_V1) {
            return Self::decrypt_v1(encoded, master_key);
        }

        Err(CryptoError::InvalidPayload)
    }

    fn encrypt_v2(plaintext: &str, master_key: &str) -> Result<String, CryptoError> {
        let mut key_bytes = derive_v2_key(master_key);
        let cipher =
            Aes256GcmSiv::new_from_slice(&key_bytes).map_err(|_| CryptoError::KeyDerivationFailed)?;
        let ciphertext = cipher
            .encrypt(GcmSivNonce::from_slice(&V2_NONCE), plaintext.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;
        key_bytes.zeroize();

        let mut payload = Vec::with_capacity(1 + ciphertext.len());
        payload.push(VERSION_V2);
        payload.extend_from_slice(&ciphertext);

        Ok(format!("{PREFIX_V2}{}", STANDARD.encode(payload)))
    }

    fn decrypt_v2(encoded: &str, master_key: &str) -> Result<String, CryptoError> {
        let payload = decode_payload(encoded, PREFIX_V2)?;

        if payload.len() < 1 + TAG_LEN {
            return Err(CryptoError::InvalidPayload);
        }

        if payload[0] != VERSION_V2 {
            return Err(CryptoError::InvalidPayload);
        }

        let ciphertext = &payload[1..];

        let mut key_bytes = derive_v2_key(master_key);
        let cipher =
            Aes256GcmSiv::new_from_slice(&key_bytes).map_err(|_| CryptoError::KeyDerivationFailed)?;
        let decrypted = cipher
            .decrypt(GcmSivNonce::from_slice(&V2_NONCE), ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        key_bytes.zeroize();

        String::from_utf8(decrypted).map_err(|_| CryptoError::DecryptionFailed)
    }

    fn encrypt_v1(plaintext: &str, master_key: &str) -> Result<String, CryptoError> {
        validate_encrypt_inputs(plaintext, master_key)?;

        let mut salt = [0_u8; SALT_LEN];
        let mut nonce = [0_u8; NONCE_LEN];

        OsRng
            .try_fill_bytes(&mut salt)
            .map_err(|_| CryptoError::RandomGenerationFailed)?;
        OsRng
            .try_fill_bytes(&mut nonce)
            .map_err(|_| CryptoError::RandomGenerationFailed)?;

        let mut key_bytes = derive_v1_key(master_key, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| CryptoError::KeyDerivationFailed)?;
        let ciphertext = cipher
            .encrypt(GcmNonce::from_slice(&nonce), plaintext.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;
        key_bytes.zeroize();

        let mut payload = Vec::with_capacity(1 + SALT_LEN + NONCE_LEN + ciphertext.len());
        payload.push(VERSION_V1);
        payload.extend_from_slice(&salt);
        payload.extend_from_slice(&nonce);
        payload.extend_from_slice(&ciphertext);

        Ok(format!("{PREFIX_V1}{}", STANDARD.encode(payload)))
    }

    fn decrypt_v1(encoded: &str, master_key: &str) -> Result<String, CryptoError> {
        let payload = decode_payload(encoded, PREFIX_V1)?;

        if payload.len() < 1 + SALT_LEN + NONCE_LEN + TAG_LEN {
            return Err(CryptoError::InvalidPayload);
        }

        if payload[0] != VERSION_V1 {
            return Err(CryptoError::InvalidPayload);
        }

        let salt_start = 1;
        let salt_end = salt_start + SALT_LEN;
        let nonce_end = salt_end + NONCE_LEN;

        let salt = &payload[salt_start..salt_end];
        let nonce = &payload[salt_end..nonce_end];
        let ciphertext = &payload[nonce_end..];

        let mut key_bytes = derive_v1_key(master_key, salt);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| CryptoError::KeyDerivationFailed)?;
        let decrypted = cipher
            .decrypt(GcmNonce::from_slice(nonce), ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        key_bytes.zeroize();

        String::from_utf8(decrypted).map_err(|_| CryptoError::DecryptionFailed)
    }
}

fn validate_encrypt_inputs(plaintext: &str, master_key: &str) -> Result<(), CryptoError> {
    if plaintext.is_empty() {
        return Err(CryptoError::EmptyInput);
    }

    if master_key.is_empty() {
        return Err(CryptoError::EmptyMasterKey);
    }

    Ok(())
}

fn decode_payload(encoded: &str, prefix: &str) -> Result<Vec<u8>, CryptoError> {
    let base64_payload = encoded
        .strip_prefix(prefix)
        .ok_or(CryptoError::InvalidPayload)?;

    STANDARD
        .decode(base64_payload)
        .map_err(|_| CryptoError::InvalidPayload)
}

fn derive_v1_key(master_key: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    pbkdf2_hmac_array::<Sha256, KEY_LEN>(master_key.as_bytes(), salt, ITERATIONS)
}

fn derive_v2_key(master_key: &str) -> [u8; KEY_LEN] {
    pbkdf2_hmac_array::<Sha256, KEY_LEN>(master_key.as_bytes(), V2_KDF_SALT, ITERATIONS)
}

#[cfg(test)]
mod tests {
    use super::{EncryptedPayload, PREFIX_V1, PREFIX_V2};

    #[test]
    fn deterministic_roundtrip_works() {
        let master_key = "correct horse battery staple";
        let plaintext = "efe.facebook";

        let encrypted = EncryptedPayload::encrypt(plaintext, master_key).unwrap();
        let decrypted = EncryptedPayload::decrypt(&encrypted, master_key).unwrap();

        assert!(encrypted.starts_with(PREFIX_V2));
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn deterministic_encryption_returns_same_output_for_same_inputs() {
        let master_key = "correct horse battery staple";
        let plaintext = "efe.facebook";

        let first = EncryptedPayload::encrypt(plaintext, master_key).unwrap();
        let second = EncryptedPayload::encrypt(plaintext, master_key).unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn different_plaintexts_produce_different_outputs() {
        let master_key = "correct horse battery staple";

        let facebook = EncryptedPayload::encrypt("efe.facebook", master_key).unwrap();
        let instagram = EncryptedPayload::encrypt("efe.instagram", master_key).unwrap();

        assert_ne!(facebook, instagram);
    }

    #[test]
    fn legacy_v1_payloads_still_decrypt() {
        let master_key = "correct horse battery staple";
        let plaintext = "efe.facebook";

        let encrypted = EncryptedPayload::encrypt_v1(plaintext, master_key).unwrap();
        let decrypted = EncryptedPayload::decrypt(&encrypted, master_key).unwrap();

        assert!(encrypted.starts_with(PREFIX_V1));
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn invalid_prefix_is_rejected() {
        let result = EncryptedPayload::decrypt("nope", "secret");
        assert!(result.is_err());
    }
}
