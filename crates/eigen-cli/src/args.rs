use crate::ANVIL_RPC_URL;
use alloy_primitives::Address;
use clap::{ArgGroup, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    about = "Eigenlayer CLI tools",
    long_about = "Tools used for AVS development purpose"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Given an initial contract address, which can be either a registry coordinator or service manager address, 
outputs addresses for all relevant Eigenlayer and AVS contracts within the network",
        name = "egnaddrs",
        group(
            ArgGroup::new("manager_or_coordinator")
                .required(true)
                .args(&["service_manager", "registry_coordinator"]),
        )
    )]
    EigenAddress {
        #[arg(
            long,
            help = "ServiceManager contract address",
            group = "manager_or_coordinator"
        )]
        service_manager: Option<Address>,

        #[arg(
            long,
            help = "BLSRegistryCoordinatorWithIndices contract address",
            group = "manager_or_coordinator"
        )]
        registry_coordinator: Option<Address>,

        #[arg(long, help = "rpc url", default_value = ANVIL_RPC_URL)]
        rpc_url: String,
    },

    #[command(about = "EigenLayer CLI Key tools", name = "egnkey")]
    EigenKey {
        #[command(subcommand)]
        subcommand: EigenKeyCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum EigenKeyCommand {
    #[command(
        about = "Generate keys for testing purpose.
This command creates ecdsa or bls key pair for testing purposes. It generates
all the relevant files for reading the key and decrypts it and also gets 
you the private keys in plaintext.

It creates the following artifacts based on arguments
- passwords.txt - contains all passwords to decrypt keys
- private_key_hex.txt - will create plaintext private keys
- keys/* - create all the encrypted json files in this folder",
        alias = "g"
    )]
    Generate {
        #[arg(long, help = "key type to create (ecdsa or bls)")]
        #[clap(value_enum)]
        key_type: KeyType,

        #[arg(long, help = "number of keys to create", default_value = "1")]
        num_keys: u32,

        #[arg(long, help = "folder to store keys")]
        output_dir: Option<String>,
    },

    #[command(
        about = "Given a private key, output its associated operatorId (hash of bn254 G1 pubkey).",
        alias = "d"
    )]
    DeriveOperatorId {
        #[arg(
            long,
            help = "(bn254) private key from which to derive operatorId from"
        )]
        private_key: String,
    },

    #[command(
        about = "Stores an ecdsa key to a file, in web3 secret storage format.",
        alias = "c"
    )]
    ConvertECDSA {
        #[arg(long, help = "private key to store (in hex)")]
        private_key: String,

        #[arg(long, help = "file to store key")]
        output_file: Option<String>,

        #[arg(long, help = "password to encrypt key")]
        password: Option<String>,
    },
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum KeyType {
    Ecdsa,
    Bls,
}
