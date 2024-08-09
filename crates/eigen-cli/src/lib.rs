#![doc(
    html_logo_url = "https://github.com/Layr-Labs/eigensdk-rs/assets/91280922/bd13caec-3c00-4afc-839a-b83d2890beb5",
    issue_tracker_base_url = "https://github.com/Layr-Labs/eigensdk-rs/issues/"
)]
pub mod args;
mod convert;
pub mod eigen_address;
mod generate;
mod operator_id;
use eth_keystore::KeystoreError;
use tokio::runtime::Runtime;

use crate::eigen_address::ContractAddresses;
use alloy_contract::Error as ContractError;
use alloy_json_rpc::RpcError;
use alloy_transport::TransportErrorKind;
use args::{Commands, EigenKeyCommand};
use ark_serialize::SerializationError;
use convert::store;
use eigen_crypto_bls::error::BlsError;
use generate::KeyGenerator;
use operator_id::derive_operator_id;
use thiserror::Error;

pub const ANVIL_RPC_URL: &str = "http://localhost:8545";

/// Possible errors raised while executing a CLI command
#[derive(Error, Debug)]
pub enum EigenCliError {
    #[error("address error")]
    EigenAddressCliError(EigenAddressCliError),
    #[error("key error")]
    EigenKeyCliError(EigenKeyCliError),
}

/// Possible errors raised while trying to get contract addresses
#[derive(Error, Debug)]
pub enum EigenAddressCliError {
    #[error("contract error")]
    ContractError(ContractError),
    #[error("RPC error")]
    RpcError(RpcError<TransportErrorKind>),
}

/// Possible errors raised while executing egnkey commands
#[derive(Error, Debug)]
pub enum EigenKeyCliError {
    #[error("file error")]
    FileError(std::io::Error),
    #[error("keystore error")]
    KeystoreError(KeystoreError),
    #[error("BLS error")]
    BLSError(BlsError),
    #[error("serialization error")]
    SerializationError(SerializationError),
}

/// Executes an `egnkey` subcommand.
///
/// # Arguments
///
/// * `subcommand` - An egnkey subcommand which can be `generate`, `convert` or `derive-operator-id`.
///
/// # Errors
///
/// - If the subcommand execution fails (`EigenKeyCliError`).
pub fn execute_egnkey_subcommand(subcommand: EigenKeyCommand) -> Result<(), EigenKeyCliError> {
    match subcommand {
        EigenKeyCommand::Generate {
            key_type,
            num_keys,
            output_dir,
        } => KeyGenerator::from(key_type).generate(num_keys, output_dir),

        EigenKeyCommand::ConvertECDSA {
            private_key,
            output_file,
            password,
        } => store(private_key.into(), output_file, password)
            .map_err(EigenKeyCliError::KeystoreError),

        EigenKeyCommand::DeriveOperatorId { private_key } => {
            let operator_id =
                derive_operator_id(private_key).map_err(EigenKeyCliError::BLSError)?;
            println!("{}", operator_id);
            Ok(())
        }
    }
}

/// Executes a CLI command.
///
/// # Arguments
///
/// * `command` - A CLI command which can be `Commands::EigenAddress` or `Commands::EigenKey`
///
/// # Errors
///
/// - If the command execution fails (`EigenCliError`).
pub fn execute_command(command: Commands) -> Result<(), EigenCliError> {
    match command {
        Commands::EigenAddress {
            service_manager,
            registry_coordinator,
            rpc_url,
        } => {
            let rt = Runtime::new().unwrap();
            let addresses = rt.block_on(async {
                ContractAddresses::get_addresses(service_manager, registry_coordinator, rpc_url)
                    .await
                    .map_err(EigenCliError::EigenAddressCliError)
            })?;
            println!("{}", serde_json::to_string_pretty(&addresses).unwrap());
            Ok(())
        }
        Commands::EigenKey { subcommand } => {
            execute_egnkey_subcommand(subcommand).map_err(EigenCliError::EigenKeyCliError)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::ANVIL_RPC_URL;
    use crate::args::EigenKeyCommand;
    use crate::convert::store;
    use crate::eigen_address::ContractAddresses;
    use crate::{
        args::{Commands, KeyType},
        execute_command,
        generate::{KeyGenerator, DEFAULT_KEY_FOLDER, PASSWORD_FILE, PRIVATE_KEY_HEX_FILE},
        operator_id::derive_operator_id,
    };
    use eigen_testing_utils::anvil_constants::{
        get_registry_coordinator_address, get_service_manager_address,
    };
    use eth_keystore::decrypt_key;
    use k256::SecretKey;
    use rstest::rstest;
    use std::fs;
    use tempfile::tempdir;

    #[rstest]
    #[case(KeyType::Ecdsa)]
    #[case(KeyType::Bls)]
    fn test_generate_key(#[case] key_type: KeyType) {
        let output_dir = tempdir().unwrap();
        let output_path = output_dir.path();
        let subcommand = EigenKeyCommand::Generate {
            key_type: key_type.clone(),
            num_keys: 1,
            output_dir: output_path.to_str().map(String::from),
        };
        let command = Commands::EigenKey { subcommand };

        execute_command(command).unwrap();

        let private_key_hex = fs::read_to_string(output_path.join(PRIVATE_KEY_HEX_FILE)).unwrap();
        let password = fs::read_to_string(output_path.join(PASSWORD_FILE)).unwrap();
        let key_name = match key_type {
            KeyType::Ecdsa => "ecdsa",
            KeyType::Bls => "bls",
        };
        let key_path = output_path
            .join(DEFAULT_KEY_FOLDER)
            .join(format!("1.{}.key.json", key_name));

        let decrypted_bytes = decrypt_key(key_path, password).unwrap();
        let decrypted_private_key = SecretKey::from_slice(&decrypted_bytes).unwrap().to_bytes();

        let private_key = hex::decode(private_key_hex).unwrap();

        assert_eq!(private_key, decrypted_private_key.as_slice());
    }

    #[test]
    fn test_egnkey_derive_operator_id() {
        let private_key =
            "1e4fa82657771dc209c466a0c2f696b39320a0284bf725cf1740971fe7e2d3cf".to_string();
        let operator_id = derive_operator_id(private_key).unwrap();
        let expected_operator_id =
            "48beccce16ccdf8000c13d5af5f91c7c3dac6c47b339d993d229af1500dbe4a9".to_string();
        assert_eq!(expected_operator_id, operator_id);
    }

    #[test]
    fn test_convert_ecdsa() {
        let private_key = KeyGenerator::random_ecdsa_key();
        let password = KeyGenerator::generate_random_password();
        let file = "key.json".to_string();
        let path = "./key.json".to_string();

        store(private_key.clone(), Some(file), Some(password.clone())).unwrap();
        let decrypted_key = decrypt_key(path.clone(), password).unwrap();
        std::fs::remove_file(path).unwrap();

        assert_eq!(private_key, decrypted_key);
    }

    #[tokio::test]
    async fn test_egnaddrs_with_service_manager_flag() {
        let service_manager_address = get_service_manager_address().await;

        let expected_addresses: ContractAddresses = serde_json::from_str(
            r#"{
            "avs": {
              "bls-apk-registry": "0x84ea74d481ee0a5332c457a4d796187f6ba67feb",
              "index-registry": "0x9e545e3c0baab3e08cdfd552c960a1050f373042",
              "registry-coordinator": "0xc3e53f4d16ae77db1c982e75a937b9f60fe63690",
              "service-manager": "0x67d269191c92caf3cd7723f116c85e6e9bf55933",
              "stake-registry": "0xa82ff9afd8f496c3d6ac40e2a0f282e47488cfc9"
            },
            "eigenlayer": {
              "delegation-manager": "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9",
              "slasher": "0xa513E6E4b8f2a923D98304ec87F64353C4D5C853",
              "strategy-manager": "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707"
            },
            "network": {
              "chain-id": "31337",
              "rpc-url": "http://localhost:8545"
            }
          }"#,
        )
        .unwrap();
        let addresses = ContractAddresses::get_addresses(
            Some(service_manager_address),
            None,
            ANVIL_RPC_URL.into(),
        )
        .await
        .unwrap();

        assert_eq!(expected_addresses, addresses);
    }

    #[tokio::test]
    async fn test_egnaddrs_with_registry_coordinator_flag() {
        let registry_coordinator_address = get_registry_coordinator_address().await;

        let expected_addresses: ContractAddresses = serde_json::from_str(
            r#"{
            "avs": {
              "bls-apk-registry": "0x84ea74d481ee0a5332c457a4d796187f6ba67feb",
              "index-registry": "0x9e545e3c0baab3e08cdfd552c960a1050f373042",
              "registry-coordinator": "0xc3e53f4d16ae77db1c982e75a937b9f60fe63690",
              "service-manager": "0x67d269191c92caf3cd7723f116c85e6e9bf55933",
              "stake-registry": "0xa82ff9afd8f496c3d6ac40e2a0f282e47488cfc9"
            },
            "eigenlayer": {
              "delegation-manager": "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9",
              "slasher": "0xa513E6E4b8f2a923D98304ec87F64353C4D5C853",
              "strategy-manager": "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707"
            },
            "network": {
              "chain-id": "31337",
              "rpc-url": "http://localhost:8545"
            }
          }"#,
        )
        .unwrap();

        let addresses = ContractAddresses::get_addresses(
            None,
            Some(registry_coordinator_address),
            ANVIL_RPC_URL.into(),
        )
        .await
        .unwrap();

        assert_eq!(expected_addresses, addresses);
    }
}
