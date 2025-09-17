use aes_gcm::{
    Aes128Gcm, Key,
    aead::{Aead, AeadCore, KeyInit, OsRng as Rng},
};
use base64::{engine::general_purpose, prelude::*};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};
use yescrypt::yescrypt_kdf;

// To hold metadata
// Seralize permits serializing to TOML
// Deserialize permits deconstruting form TOML
// Debug to print the structure
#[derive(Serialize, Deserialize, Debug)]
pub struct CryptoMetadata {
    salt1: [u8; 16],
    salt2: [u8; 16],
    salt3: [u8; 16],
    nonce: Option<Base64>,
    dgst: Base64,
}
impl CryptoMetadata {
    fn new(salt1: [u8; 16], salt2: [u8; 16], salt3: [u8; 16], dgst: Base64) -> Self {
        Self {
            salt1,
            salt2,
            salt3,
            dgst,
            nonce: None,
        }
    }
    fn set_nonce(&mut self, nonce: Base64) {
        self.nonce = Some(nonce)
    }
}

/// Marker for Base64 encoding in JSON
// I needed to derive these so that CryptoMetadata is valid
#[derive(Serialize, Deserialize, Debug)]
struct Base64(String);

fn keyder(password: &[u8]) -> (Vec<u8>, CryptoMetadata) {
    // Generate the Master Key and its salt
    let salt1 = [0u8; 16]; // 16-byte (128-bit) salt
    let mk = derive_b64(password, salt1, 0xB6, 32768, 32, 1, 0, 0, 16);

    // Generate the digset of the MK to store it, and its salt
    let salt2 = [0u8; 16]; // 16-byte (128-bit) salt
    let dgst = derive_b64(&mk, salt2, 0xB6, 4096, 32, 1, 0, 0, 16);

    // Code the MK's digest in Base64
    let dgst_b64 = Base64(general_purpose::STANDARD.encode(&dgst));

    // Generate the DPK to use it to encrypt
    let salt3 = [0u8; 16]; // 16-byte (128-bit) salt
    let dpk = derive_b64(&mk, salt3, 0xB6, 32768, 32, 1, 0, 0, 16);

    // Save the metadata to file
    let metadata = CryptoMetadata::new(salt1, salt2, salt3, dgst_b64);

    (dpk, metadata)
}

fn derive_b64(
    password: &[u8],
    mut salt: [u8; 16],
    flags: u32,
    n: u64,
    r: u32,
    p: u32,
    t: u32,
    g: u32,
    dstlen: usize,
) -> Vec<u8> {
    // fill with CSPRNG; the beginning of the line is to contain the Err that may generate.
    let _ = OsRng.try_fill_bytes(&mut salt);

    // Derive
    let key = yescrypt_kdf(&password, &salt, flags, n, r, p, t, g, dstlen);

    // Return
    key
}

pub fn encrypt_archive(encr_archive_path: &Path, archive_path: &Path, password: &[u8]) -> Vec<u8> {
    // Get the DPK, and the metadata to write it off
    let (dpk, mut metadata) = keyder(password);

    // Turn our key to the format that the function accepts
    let key = Key::<Aes128Gcm>::from_slice(&dpk);

    // Set the cipher function
    let cipher = Aes128Gcm::new(&key);

    // Generate the nonce
    let nonce = Aes128Gcm::generate_nonce(&mut Rng); // 96-bits; unique per message

    // Add the nonce to the metadata, in base64 encoding
    metadata.set_nonce(Base64(general_purpose::STANDARD.encode(&nonce)));

    // Turning the string to TOML format
    let toml_string = toml::to_string(&metadata).unwrap();

    // Write off the metadata
    fs::write("test/.metadata", toml_string).unwrap();

    // Read the file before encrypting it
    let mut plaintext = Vec::new();
    let mut file = File::open(archive_path).unwrap();
    file.read_to_end(&mut plaintext).unwrap();

    // Encipher the archive
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).unwrap();

    // write the cipher
    fs::write(&encr_archive_path, &ciphertext).unwrap();

    // Return file content
    plaintext
}
