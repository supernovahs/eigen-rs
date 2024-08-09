//! register operator in quorum with avs registry coordinator
use alloy_primitives::U256;
use alloy_primitives::{Bytes, FixedBytes};
use alloy_signer_local::PrivateKeySigner;
use eigen_client_avsregistry::writer::AvsRegistryChainWriter;
use eigen_client_elcontracts::reader::ELChainReader;
use eigen_client_elcontracts::writer::ELChainWriter;
use eigen_crypto_bls::BlsKeyPair;
use eigen_logging::get_test_logger;
use eigen_testing_utils::m2_holesky_constants::{
    AVS_DIRECTORY_ADDRESS, DELEGATION_MANAGER_ADDRESS, OPERATOR_STATE_RETRIEVER,
    REGISTRY_COORDINATOR, SLASHER_ADDRESS, STRATEGY_MANAGER_ADDRESS,
};
use eigen_types::operator::Operator;
use eyre::Result;
use std::str::FromStr;
#[tokio::main]
async fn main() -> Result<()> {
    let holesky_provider = "https://ethereum-holesky.blockpi.network/v1/rpc/public";
    let pvt_key = "bead471191bea97fc3aeac36c9d74c895e8a6242602e144e43152f96219e96e8";
    let test_logger = get_test_logger();
    let avs_registry_writer = AvsRegistryChainWriter::build_avs_registry_chain_writer(
        test_logger.clone(),
        holesky_provider.to_string(),
        pvt_key.to_string(),
        REGISTRY_COORDINATOR,
        OPERATOR_STATE_RETRIEVER,
    )
    .await
    .expect("avs writer build fail ");
    let bls_key_pair = BlsKeyPair::new(
        "12248929636257230549931416853095037629726205319386239410403476017439825112537".to_string(),
    )
    .unwrap();

    let digest_hash: FixedBytes<32> = FixedBytes::from([
        0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
        0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
        0x02, 0x02,
    ]);
    let sig_expiry: U256 = U256::from(1723123419);
    let quorum_nums = Bytes::from([0x01]);

    let el_chain_reader = ELChainReader::new(
        get_test_logger().clone(),
        SLASHER_ADDRESS,
        DELEGATION_MANAGER_ADDRESS,
        AVS_DIRECTORY_ADDRESS,
        "https://ethereum-holesky.blockpi.network/v1/rpc/public".to_string(),
    );
    let el_writer = ELChainWriter::new(
        DELEGATION_MANAGER_ADDRESS,
        STRATEGY_MANAGER_ADDRESS,
        el_chain_reader,
        "https://ethereum-holesky.blockpi.network/v1/rpc/public".to_string(),
        "bead471191bea97fc3aeac36c9d74c895e8a6242602e144e43152f96219e96e8".to_string(),
    );
    let wallet = PrivateKeySigner::from_str(
        "bead471191bea97fc3aeac36c9d74c895e8a6242602e144e43152f96219e96e8",
    )
    .expect("no key ");
    let operator_details = Operator::new(
        wallet.address(),
        wallet.address(),
        wallet.address(),
        3,
        Some("eigensdk-rs".to_string()),
    );
    let _s = el_writer.register_as_operator(operator_details).await;

    avs_registry_writer
        .register_operator_in_quorum_with_avs_registry_coordinator(
            bls_key_pair,
            digest_hash,
            sig_expiry,
            quorum_nums,
            "65.109.158.181:33078;31078".to_string(),
        )
        .await
        .unwrap();
    Ok(())
}
