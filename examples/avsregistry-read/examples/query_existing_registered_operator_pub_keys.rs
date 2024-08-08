//! query existing registered operator pub keys for a specific block range
use eigen_client_avsregistry::reader::AvsRegistryChainReader;
use eigen_logging::get_test_logger;
use eigen_testing_utils::m2_holesky_constants::{OPERATOR_STATE_RETRIEVER, REGISTRY_COORDINATOR};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let holesky_provider = "https://holesky.drpc.org";
    let avs_registry = AvsRegistryChainReader::new(
        get_test_logger().clone(),
        REGISTRY_COORDINATOR,
        OPERATOR_STATE_RETRIEVER,
        holesky_provider.to_string(),
    )
    .await
    .expect("failed to build avs registry chain reader");
    let start_block = 1722400;
    let to_block = 1748590;
    let operators_state = avs_registry
        .query_existing_registered_operator_pub_keys(
            start_block,
            to_block,
            holesky_provider.to_string(),
        )
        .await
        .unwrap();

    println!(
        "operator state from block: {:?} to block: {:?} is {:?}",
        start_block, to_block, operators_state
    );

    Ok(())
}
