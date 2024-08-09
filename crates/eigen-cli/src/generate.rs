use crate::args::KeyType;
use crate::EigenKeyCliError;
use ark_ff::UniformRand;
use ark_serialize::{CanonicalSerialize, SerializationError};
use eth_keystore::encrypt_key;
use rand::{distributions::Alphanumeric, Rng};
use rand_core::OsRng;
use std::io::Write;
use std::{
    fs::{self, File},
    path::Path,
};
use uuid::Uuid;

const PASSWORD_LENGTH: usize = 20;
pub const DEFAULT_KEY_FOLDER: &str = "keys";
pub const PASSWORD_FILE: &str = "password.txt";
pub const PRIVATE_KEY_HEX_FILE: &str = "private_key_hex.txt";

pub enum KeyGenerator {
    ECDSAKeyGenerator,
    BLSKeyGenerator,
}

impl KeyGenerator {
    /// Generates a number of private keys in the given directory.
    ///
    /// # Arguments
    ///
    /// * `num_keys` - The number of keys to generate.
    /// * `output_dir` - The directory where the key files are generated.
    pub fn generate(
        self,
        num_keys: u32,
        output_dir: Option<String>,
    ) -> Result<(), EigenKeyCliError> {
        let dir_name = match output_dir {
            None => {
                let id = Uuid::new_v4();
                format!("{}-{}", self.key_name(), id)
            }
            Some(dir) => dir,
        };
        let dir_path = Path::new(&dir_name);
        let key_path = dir_path.join(DEFAULT_KEY_FOLDER);
        fs::create_dir_all(&key_path).map_err(EigenKeyCliError::FileError)?;

        self.generate_keys(num_keys, dir_path)
    }

    /// Generates a number of private keys and stores them both encrypted and in plaintext.
    /// It creates the following files:
    /// - `passwords.txt`: contains all passwords to decrypt keys
    /// - `private_key_hex.txt`: plaintext private keys
    /// - `keys/*`: all the encrypted json files in this folder
    ///
    /// # Arguments
    ///
    /// * `num_keys` - The number of keys to generate.
    /// * `path` - The path to the directory where the generated files are stored.
    fn generate_keys(self, num_keys: u32, path: &Path) -> Result<(), EigenKeyCliError> {
        let key_path = path.join(DEFAULT_KEY_FOLDER);
        let private_key_path = path.join(PRIVATE_KEY_HEX_FILE);
        let password_path = path.join(PASSWORD_FILE);

        for i in 0..num_keys {
            let password = KeyGenerator::generate_random_password();
            let private_key = self
                .random_key()
                .map_err(EigenKeyCliError::SerializationError)?;
            let private_key_hex = hex::encode(private_key.clone());

            // encrypt the private key into `path` directory
            let name = format!("{}.{}.key.json", i + 1, self.key_name());
            encrypt_key(
                key_path.clone(),
                &mut OsRng,
                private_key,
                password.clone(),
                Some(&name),
            )
            .map_err(EigenKeyCliError::KeystoreError)?;

            // write the private key into `private_key_file`
            File::create(private_key_path.clone())
                .and_then(|mut file| file.write_all(private_key_hex.as_bytes()))
                .map_err(EigenKeyCliError::FileError)?;

            // write the password into `password_file`
            File::create(password_path.clone())
                .and_then(|mut file| file.write_all(password.as_bytes()))
                .map_err(EigenKeyCliError::FileError)?;

            if (i + 1) % 50 == 0 {
                println!("Generated {} keys\n", i + 1);
            }
        }
        Ok(())
    }

    /// Generates a random key which can be type ecdsa or BLS.
    ///
    /// # Returns
    ///
    /// * A private key as a vector of bytes.
    fn random_key(&self) -> Result<Vec<u8>, SerializationError> {
        match self {
            KeyGenerator::ECDSAKeyGenerator => Ok(Self::random_ecdsa_key()),
            KeyGenerator::BLSKeyGenerator => Ok(Self::random_bls_key()?),
        }
    }

    /// Generates a random ecdsa key.
    ///
    /// # Returns
    ///
    /// * An ecdsa private key as a vector of bytes.
    pub fn random_ecdsa_key() -> Vec<u8> {
        let private_key = k256::SecretKey::random(&mut OsRng);
        private_key.to_bytes().as_slice().to_vec()
    }

    /// Generates a random BLS key.
    ///
    /// # Returns
    ///
    /// * A BLS private key as a vector of bytes.
    fn random_bls_key() -> Result<Vec<u8>, SerializationError> {
        let mut buffer = Vec::new();
        let private_key = eigen_crypto_bls::PrivateKey::rand(&mut OsRng);
        private_key.serialize_uncompressed(&mut buffer)?;
        Ok(buffer)
    }

    /// Generates a 20-character random password.
    ///
    /// # Returns
    ///
    /// * A random password.
    pub fn generate_random_password() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(PASSWORD_LENGTH)
            .map(char::from)
            .collect()
    }

    /// Get the key type.
    ///
    /// # Returns
    ///
    /// * The key type as a string.
    fn key_name(&self) -> String {
        match self {
            KeyGenerator::ECDSAKeyGenerator => "ecdsa",
            KeyGenerator::BLSKeyGenerator => "bls",
        }
        .to_string()
    }
}

impl From<KeyType> for KeyGenerator {
    fn from(value: KeyType) -> Self {
        match value {
            KeyType::Ecdsa => KeyGenerator::ECDSAKeyGenerator,
            KeyType::Bls => KeyGenerator::BLSKeyGenerator,
        }
    }
}
