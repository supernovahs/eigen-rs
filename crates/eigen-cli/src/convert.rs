use eth_keystore::{encrypt_key, KeystoreError};
use rand_core::OsRng;
use std::path::Path;

const DEFAULT_KEYSTORE_NAME: &str = "key.json";

/// Stores an ecdsa key to a file, in web3 secret storage format.
///
/// # Arguments
///
/// * `private_key` - A private key to store.
/// * `output_file` - The name of the file where the key is going to be stored.
/// * `password` - The password used to encrypt the key. *Note:* If `password` is `None` then the empty string is used as password.
///
/// # Errors
///
/// - If the key encryption fails.
pub fn store(
    private_key: Vec<u8>,
    output_file: Option<String>,
    password: Option<String>,
) -> Result<(), KeystoreError> {
    let dir = Path::new(".");

    encrypt_key(
        dir,
        &mut OsRng,
        private_key,
        password.unwrap_or_default(),
        Some(&output_file.unwrap_or(DEFAULT_KEYSTORE_NAME.into())),
    )?;

    Ok(())
}
