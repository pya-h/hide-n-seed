pub mod encryptor {
    use aes_gcm::aead::{Aead, AeadCore, OsRng};
    use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
    use hex::{decode, encode};
    use std::io;
    use std::result::Result::Err;

    const NONCE_LENGTH: usize = 12;

    pub fn string_to_fixed_array(input: &str) -> [u8; 32] {
        let bytes = input.as_bytes();
        let mut array = [0u8; 32];
        let len = bytes.len().min(32);
        array[..len].copy_from_slice(&bytes[..len]);
        array
    }

    pub fn encrypt<T: AsRef<[u8]>>(input: T, key: &[u8; 32]) -> Result<(String, String), io::Error> {
        let input_bytes = input.as_ref();
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        match cipher.encrypt(&nonce, input_bytes) {
            Ok(ciphertext) => Ok((encode(nonce), encode(ciphertext))),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
        }
    }

    pub fn decrypt(
        nonce_hex: &str,
        ciphertext_hex: &str,
        key: &[u8; 32],
    ) -> Result<String, io::Error> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        match decode(nonce_hex) {
            Ok(decoded_nonce) => {
                let nonce = Nonce::from_slice(&decoded_nonce);
                return match decode(ciphertext_hex) {
                    Ok(decoded_ciphertext) => {
                        return match cipher.decrypt(nonce, decoded_ciphertext[..].as_ref()) {
                            Ok(decrypted_bytes) => {
                                if let Ok(decrypted_string) = String::from_utf8(decrypted_bytes) {
                                    Ok(decrypted_string)
                                } else {
                                    Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))
                                }
                            }
                            Err(e) => {
                                Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
                            }
                        };
                    }
                    Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
                };
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
        }
    }

    pub fn decrypt_as_bytes(
        nonce_hex: &str,
        ciphertext_hex: &str,
        key: &[u8; 32],
    ) -> Result<Vec<u8>, io::Error> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        match decode(nonce_hex) {
            Ok(decoded_nonce) => {
                let nonce = Nonce::from_slice(&decoded_nonce);
                return match decode(ciphertext_hex) {
                    Ok(decoded_ciphertext) => {
                        return cipher
                            .decrypt(nonce, decoded_ciphertext[..].as_ref())
                            .map_err(|err| {
                                io::Error::new(io::ErrorKind::InvalidInput, err.to_string())
                            });
                    }
                    Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
                };
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
        }
    }

    pub fn separate_nonce_n_password(encrypted_nonce_n_pass: &[u8]) -> (String, String) {
        let (nonce_as_bytes, ciphered_text_as_bytes) =
            encrypted_nonce_n_pass.split_at(NONCE_LENGTH);
        (encode(nonce_as_bytes), encode(ciphered_text_as_bytes))
    }
}
