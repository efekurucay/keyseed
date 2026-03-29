use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use pbkdf2::pbkdf2_hmac_array;
use rand::{rngs::OsRng, RngCore};
use sha2::Sha256;
use thiserror::Error;
use zeroize::Zeroize;

const PREFIX: &str = "hashit:v1:";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const TAG_LEN: usize = 16;
const ITERATIONS: u32 = 210_000;

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
        if plaintext.is_empty() {
            return Err(CryptoError::EmptyInput);
        }

        if master_key.is_empty() {
            return Err(CryptoError::EmptyMasterKey);
        }

        let mut salt = [0_u8; SALT_LEN];
        let mut nonce = [0_u8; NONCE_LEN];

        OsRng
            .try_fill_bytes(&mut salt)
            .map_err(|_| CryptoError::RandomGenerationFailed)?;
        OsRng
            .try_fill_bytes(&mut nonce)
            .map_err(|_| CryptoError::RandomGenerationFailed)?;

        let mut key_bytes = derive_key(master_key, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| CryptoError::KeyDerivationFailed)?;
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;
        key_bytes.zeroize();

        let mut payload = Vec::with_capacity(1 + SALT_LEN + NONCE_LEN + ciphertext.len());
        payload.push(VERSION);
        payload.extend_from_slice(&salt);
        payload.extend_from_slice(&nonce);
        payload.extend_from_slice(&ciphertext);

        Ok(format!("{PREFIX}{}", STANDARD.encode(payload)))
    }

    pub fn decrypt(encoded: &str, master_key: &str) -> Result<String, CryptoError> {
        let encoded = encoded.trim();

        if encoded.is_empty() {
            return Err(CryptoError::EmptyInput);
        }

        if master_key.is_empty() {
            return Err(CryptoError::EmptyMasterKey);
        }

        if !encoded.starts_with(PREFIX) {
            return Err(CryptoError::InvalidPayload);
        }

        let base64_payload = &encoded[PREFIX.len()..];
        let payload = STANDARD
            .decode(base64_payload)
            .map_err(|_| CryptoError::InvalidPayload)?;

        if payload.len() < 1 + SALT_LEN + NONCE_LEN + TAG_LEN {
            return Err(CryptoError::InvalidPayload);
        }

        if payload[0] != VERSION {
            return Err(CryptoError::InvalidPayload);
        }

        let salt_start = 1;
        let salt_end = salt_start + SALT_LEN;
        let nonce_end = salt_end + NONCE_LEN;

        let salt = &payload[salt_start..salt_end];
        let nonce = &payload[salt_end..nonce_end];
        let ciphertext = &payload[nonce_end..];

        let mut key_bytes = derive_key(master_key, salt);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| CryptoError::KeyDerivationFailed)?;
        let decrypted = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        key_bytes.zeroize();

        String::from_utf8(decrypted).map_err(|_| CryptoError::DecryptionFailed)
    }
}

fn derive_key(master_key: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    pbkdf2_hmac_array::<Sha256, KEY_LEN>(master_key.as_bytes(), salt, ITERATIONS)
}

#[cfg(test)]
mod tests {
    use super::{EncryptedPayload, PREFIX};

    #[test]
    fn roundtrip_works() {
        let master_key = "correct horse battery staple";
        let plaintext = "merhaba dünya";

        let encrypted = EncryptedPayload::encrypt(plaintext, master_key).unwrap();
        let decrypted = EncryptedPayload::decrypt(&encrypted, master_key).unwrap();

        assert!(encrypted.starts_with(PREFIX));
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn invalid_prefix_is_rejected() {
        let result = EncryptedPayload::decrypt("nope", "secret");
        assert!(result.is_err());
    }
}
