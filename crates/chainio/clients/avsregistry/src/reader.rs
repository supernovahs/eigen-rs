use crate::error::AvsRegistryError;
use alloy_primitives::{Address, Bytes, FixedBytes, B256, U256};
use alloy_provider::Provider;
use alloy_rpc_types::Filter;
use ark_ff::Zero;
use eigen_types::operator::{bitmap_to_quorum_ids, OperatorPubKeys};
use eigen_utils::{
    binding::{BLSApkRegistry, OperatorStateRetriever, RegistryCoordinator, StakeRegistry},
    get_provider,
};
use num_bigint::BigInt;
use std::collections::HashMap;
use std::fmt::Debug;
use tracing::debug;

/// Avs Registry chainreader
#[derive(Debug, Clone, Default)]
pub struct AvsRegistryChainReader {
    bls_apk_registry_addr: Address,
    registry_coordinator_addr: Address,
    operator_state_retriever: Address,
    stake_registry_addr: Address,
    provider: String,
}

trait AvsRegistryReader {
    fn get_quorum_count() -> Result<u8, String>;
}

impl AvsRegistryChainReader {
    /// New AvsRegistryChainReader instance
    pub async fn new(
        registry_coordinator_addr: Address,
        operator_state_retriever_addr: Address,
        provider_url: String,
    ) -> Result<AvsRegistryChainReader, AvsRegistryError> {
        let provider = get_provider(&provider_url);

        let contract_registry_coordinator =
            RegistryCoordinator::new(registry_coordinator_addr, &provider);

        let bls_apk_registry_addr_result =
            contract_registry_coordinator.blsApkRegistry().call().await;

        match bls_apk_registry_addr_result {
            Ok(bls_apk_registry_return) => {
                let RegistryCoordinator::blsApkRegistryReturn {
                    _0: bls_apk_registry_addr,
                } = bls_apk_registry_return;

                let stake_registry_addr_result =
                    contract_registry_coordinator.stakeRegistry().call().await;

                match stake_registry_addr_result {
                    Ok(stake_registry_return) => {
                        let RegistryCoordinator::stakeRegistryReturn {
                            _0: stake_registry_addr,
                        } = stake_registry_return;

                        Ok(AvsRegistryChainReader {
                            bls_apk_registry_addr,
                            registry_coordinator_addr,
                            operator_state_retriever: operator_state_retriever_addr,
                            stake_registry_addr,
                            provider: provider_url.clone(),
                        })
                    }
                    Err(_) => Err(AvsRegistryError::GetStakeRegistry),
                }
            }

            Err(_) => Err(AvsRegistryError::GetBlsApkRegistry),
        }
    }

    /// Get quorum count
    pub async fn get_quorum_count(&self) -> Result<u8, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let quorum_count_result = contract_registry_coordinator.quorumCount().call().await;

        match quorum_count_result {
            Ok(quorum_count) => {
                let RegistryCoordinator::quorumCountReturn { _0: quorum } = quorum_count;
                Ok(quorum)
            }

            Err(_) => Err(AvsRegistryError::GetQuorumCount),
        }
    }

    /// Get operators stake in quorums at a particular block
    pub async fn get_operators_stake_in_quorums_at_block(
        &self,
        block_number: u32,
        quorum_numbers: Bytes,
    ) -> Result<Vec<Vec<OperatorStateRetriever::Operator>>, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let contract_operator_state_retriever =
            OperatorStateRetriever::new(self.operator_state_retriever, provider);
        let operator_state_result = contract_operator_state_retriever
            .getOperatorState_0(self.registry_coordinator_addr, quorum_numbers, block_number)
            .call()
            .await;

        match operator_state_result {
            Ok(operator_state) => {
                let OperatorStateRetriever::getOperatorState_0Return { _0: quorum } =
                    operator_state;
                Ok(quorum)
            }
            Err(_) => Err(AvsRegistryError::GetOperatorState),
        }
    }

    /// Get operators stake in quorums at block operator id
    pub async fn get_operators_stake_in_quorums_at_block_operator_id(
        &self,
        block_number: u32,
        operator_id: B256,
    ) -> Result<(U256, Vec<Vec<OperatorStateRetriever::Operator>>), AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let contract_operator_state_retriever =
            OperatorStateRetriever::new(self.operator_state_retriever, provider);
        let operator_state_with_registry_coordinator_and_oeprator_id_result =
            contract_operator_state_retriever
                .getOperatorState_1(self.registry_coordinator_addr, operator_id, block_number)
                .call()
                .await;

        match operator_state_with_registry_coordinator_and_oeprator_id_result {
            Ok(operator_state_with_registry_coordinator_and_oeprator_id) => {
                let OperatorStateRetriever::getOperatorState_1Return {
                    _0: stake,
                    _1: operator_state,
                } = operator_state_with_registry_coordinator_and_oeprator_id;
                Ok((stake, operator_state))
            }
            Err(_) => Err(AvsRegistryError::GetOperatorStateWithRegistryCoordinatorAndOperatorId),
        }
    }

    /// Get operators stake in quorums at current block
    pub async fn get_operators_stake_in_quorums_at_current_block(
        &self,
        quorum_numbers: Bytes,
    ) -> Result<Vec<Vec<OperatorStateRetriever::Operator>>, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let current_block_number_result = provider.get_block_number().await;

        match current_block_number_result {
            Ok(current_block_number) => {
                if current_block_number > u32::MAX.into() {
                    return Err(AvsRegistryError::BlockNumberOverflow);
                }

                let operators_stake_in_quorums_at_block_result = self
                    .get_operators_stake_in_quorums_at_block(
                        current_block_number as u32,
                        quorum_numbers,
                    )
                    .await;

                match operators_stake_in_quorums_at_block_result {
                    Ok(operators_stake_in_quorums_at_block) => {
                        Ok(operators_stake_in_quorums_at_block)
                    }
                    Err(_) => Err(AvsRegistryError::GetOperatorStakeInQuorumAtBlockNumber),
                }
            }
            Err(_) => Err(AvsRegistryError::GetBlockNumber),
        }
    }

    /// Get operators stake in quorums of operator at block
    pub async fn get_operators_stake_in_quorums_of_operator_at_block(
        &self,
        operator_id: B256,
        block_number: u32,
    ) -> Result<(Vec<u8>, Vec<Vec<OperatorStateRetriever::Operator>>), AvsRegistryError> {
        let result_ = self
            .get_operators_stake_in_quorums_at_block_operator_id(block_number, operator_id)
            .await;

        match result_ {
            Ok((quorum_bitmaps, operator_stakes)) => {
                let quorums = bitmap_to_quorum_ids(quorum_bitmaps);

                let s = (quorums, operator_stakes);
                Ok(s)
            }
            Err(_) => Err(AvsRegistryError::GetOperatorStakeInQuorumAtBlockOperatorId),
        }
    }

    /// Get operators stake in quorums of operator at current block
    pub async fn get_operators_stake_in_quorums_of_operator_at_current_block(
        &self,
        operator_id: B256,
    ) -> Result<(Vec<u8>, Vec<Vec<OperatorStateRetriever::Operator>>), AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let current_block_number_result = provider.get_block_number().await;

        match current_block_number_result {
            Ok(current_block_number) => {
                if current_block_number > u32::MAX.into() {
                    return Err(AvsRegistryError::BlockNumberOverflow);
                }

                let operator_stake_in_quorum_of_operaotr_at_block_result = self
                    .get_operators_stake_in_quorums_of_operator_at_block(
                        operator_id,
                        current_block_number as u32,
                    )
                    .await;

                match operator_stake_in_quorum_of_operaotr_at_block_result {
                    Ok(operator_stake_in_quorum_of_operaotr_at_block) => {
                        Ok(operator_stake_in_quorum_of_operaotr_at_block)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(AvsRegistryError::AlloyContractError(
                alloy_contract::Error::TransportError(e),
            )),
        }
    }

    /// Get operator stake in quorums of operator at current block
    pub async fn get_operator_stake_in_quorums_of_operator_at_current_block(
        &self,
        operator_id: B256,
    ) -> Result<HashMap<u8, BigInt>, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, &provider);

        let quorum_bitmap = registry_coordinator
            .getCurrentQuorumBitmap(operator_id)
            .call()
            .await?;

        let RegistryCoordinator::getCurrentQuorumBitmapReturn { _0: quo } = quorum_bitmap;

        let quorums = bitmap_to_quorum_ids(quo);

        let mut quorum_stakes: HashMap<u8, BigInt> = HashMap::new();

        let stake_registry = StakeRegistry::new(self.stake_registry_addr, &provider);
        for quorum in quorums.iter() {
            let stakes_result = stake_registry
                .getCurrentStake(operator_id, *quorum)
                .call()
                .await?;

            let StakeRegistry::getCurrentStakeReturn { _0: c_stake } = stakes_result;
            quorum_stakes.insert(*quorum, c_stake.into());
        }
        Ok(quorum_stakes)
    }

    /// Get Signature indices
    pub async fn get_check_signatures_indices(
        &self,
        reference_block_number: u32,
        quorum_numbers: Vec<u8>,
        non_signer_operator_ids: Vec<FixedBytes<32>>,
    ) -> Result<OperatorStateRetriever::CheckSignaturesIndices, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let contract_operator_state_retriever =
            OperatorStateRetriever::new(self.operator_state_retriever, provider);

        let check_signature_indices = contract_operator_state_retriever
            .getCheckSignaturesIndices(
                self.registry_coordinator_addr,
                reference_block_number,
                quorum_numbers.into(),
                non_signer_operator_ids,
            )
            .call()
            .await?;
        let OperatorStateRetriever::getCheckSignaturesIndicesReturn { _0: indices } =
            check_signature_indices;
        Ok(indices)
    }

    /// Get Operator Id
    pub async fn get_operator_id(
        &self,
        operator_address: Address,
    ) -> Result<FixedBytes<32>, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let operator_id_return = contract_registry_coordinator
            .getOperatorId(operator_address)
            .call()
            .await?;
        let RegistryCoordinator::getOperatorIdReturn { _0: operator_id } = operator_id_return;
        Ok(operator_id)
    }

    /// Get Operator from operator id
    pub async fn get_operator_from_id(
        &self,
        operator_id: [u8; 32],
    ) -> Result<Address, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, &provider);

        let operator_address_return = contract_registry_coordinator
            .getOperatorFromId(operator_id.into())
            .call()
            .await?;

        let RegistryCoordinator::getOperatorFromIdReturn {
            _0: operator_address,
        } = operator_address_return;
        Ok(operator_address)
    }

    /// Check if operator is registered
    pub async fn is_operator_registered(
        &self,
        operator_address: Address,
    ) -> Result<bool, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let operator_status_return = contract_registry_coordinator
            .getOperatorStatus(operator_address)
            .call()
            .await?;

        let RegistryCoordinator::getOperatorStatusReturn {
            _0: operator_status,
        } = operator_status_return;

        Ok(operator_status == 1)
    }

    /// Queies existing operators from for a particular block range
    /// Queies existing operators from for a particular block range
    pub async fn query_existing_registered_operator_pub_keys(
        &self,
        start_block: u64,
        mut stop_block: u64,
    ) -> Result<(Vec<Address>, Vec<OperatorPubKeys>), AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let query_block_range = 1024;
        let current_block_number_result = provider.get_block_number().await;

        match current_block_number_result {
            Ok(current_block_number) => {
                if stop_block.is_zero() {
                    stop_block = current_block_number;
                }
                let mut i = start_block;
                let mut operator_addresses: Vec<Address> = vec![];
                let mut operator_pub_keys: Vec<OperatorPubKeys> = vec![];
                while i <= stop_block {
                    let mut to_block = i + (query_block_range - 1);
                    if to_block > stop_block {
                        to_block = stop_block;
                    }
                    let filter = Filter::new()
                        .select(i..to_block)
                        .event("NewPubkeyRegistration(address,(uint256,uint256),(uint256[2],uint256[2]))")
                        .address(self.bls_apk_registry_addr);

                    let logs_result = provider.get_logs(&filter).await;

                    match logs_result {
                        Ok(logs) => {
                            debug!(transactionLogs = ?logs, "avsRegistryChainReader.QueryExistingRegisteredOperatorPubKeys");

                            for v_log in logs.iter() {
                                if let Ok(pub_key_reg) =
                                    v_log.log_decode::<BLSApkRegistry::NewPubkeyRegistration>()
                                {
                                    let data = pub_key_reg.data();
                                    let operator_addr = data.operator;
                                    operator_addresses.push(operator_addr);
                                    let g1_pub_key = data.pubkeyG1.clone();
                                    let g2_pub_key = data.pubkeyG2.clone();

                                    let operator_pub_key = OperatorPubKeys {
                                        g1_pub_key,
                                        g2_pub_key,
                                    };

                                    operator_pub_keys.push(operator_pub_key);
                                }
                            }
                        }
                        Err(e) => {
                            return Err(AvsRegistryError::AlloyContractError(
                                alloy_contract::Error::TransportError(e),
                            ))
                        }
                    }
                    i += query_block_range;
                }

                Ok((operator_addresses, operator_pub_keys))
            }
            Err(e) => Err(AvsRegistryError::AlloyContractError(
                alloy_contract::Error::TransportError(e),
            )),
        }
    }

    /// Query existing operator sockets
    pub async fn query_existing_registered_operator_sockets(
        &self,
        start_block: u64,
        stop_block: u64,
    ) -> Result<HashMap<FixedBytes<32>, String>, AvsRegistryError> {
        let provider = get_provider(&self.provider);

        let mut operator_id_to_socket = HashMap::new();

        let query_block_range = 10000;

        let mut i = start_block;

        while i <= stop_block {
            let mut to_block = i + (query_block_range - 1);
            if to_block > stop_block {
                to_block = stop_block;
            }

            let mut filter = Filter::new()
                .select(start_block..to_block)
                .event("OperatorSocketUpdate(bytes32,string)")
                .address(self.registry_coordinator_addr);
            if stop_block == 0 {
                let current_block_number_result = provider.get_block_number().await;

                match current_block_number_result {
                    Ok(current_block_number) => {
                        filter = filter.clone().select(start_block..current_block_number);
                    }

                    Err(e) => {
                        return Err(AvsRegistryError::AlloyContractError(
                            alloy_contract::Error::TransportError(e),
                        ))
                    }
                }
            };

            let logs_result = provider.get_logs(&filter).await;

            match logs_result {
                Ok(logs) => {
                    for v_log in logs.iter() {
                        let socket_update_filter_option = v_log
                            .log_decode::<RegistryCoordinator::OperatorSocketUpdate>()
                            .ok();
                        if let Some(socket_update_filter) = socket_update_filter_option {
                            let data = socket_update_filter.data();
                            let operator_id = data.operatorId;
                            let socket = &data.socket;
                            operator_id_to_socket.insert(operator_id, socket.clone());
                        }
                    }

                    i += query_block_range;
                }
                Err(e) => {
                    return Err(AvsRegistryError::AlloyContractError(
                        alloy_contract::Error::TransportError(e),
                    ))
                }
            }
        }
        Ok(operator_id_to_socket)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use hex::FromHex;
    use std::str::FromStr;
    const HOLESKY_REGISTRY_COORDINATOR: &str = "0x53012C69A189cfA2D9d29eb6F19B32e0A2EA3490";
    const HOLESKY_OPERATOR_STATE_RETRIEVER: &str = "0xB4baAfee917fb4449f5ec64804217bccE9f46C67";
    const HOLESKY_STAKE_REGISTRY: &str = "0xBDACD5998989Eec814ac7A0f0f6596088AA2a270";
    const HOLESKY_BLS_APK_REGISTRY: &str = "0x066cF95c1bf0927124DFB8B02B401bc23A79730D";

    async fn build_avs_registry_chain_reader() -> AvsRegistryChainReader {
        let holesky_registry_coordinator =
            Address::from_str(HOLESKY_REGISTRY_COORDINATOR).expect("failed to parse address");
        let holesky_operator_state_retriever =
            Address::from_str(HOLESKY_OPERATOR_STATE_RETRIEVER).expect("failed to parse address");

        let holesky_provider = "https://ethereum-holesky.blockpi.network/v1/rpc/public";
        AvsRegistryChainReader::new(
            holesky_registry_coordinator,
            holesky_operator_state_retriever,
            holesky_provider.to_string(),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_get_quorum_count() {
        let avs_reader = build_avs_registry_chain_reader().await;

        let _ = avs_reader.get_quorum_count().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_operators_stake_in_quorums_at_block() {
        let avs_reader = build_avs_registry_chain_reader().await;

        let quorum_number = Bytes::from_hex("0x00").expect("bytes parse");
        let _ = avs_reader
            .get_operators_stake_in_quorums_at_block(1245063, quorum_number)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_operators_stake_in_quorums_at_block_operator_id() {
        let avs_reader = build_avs_registry_chain_reader().await;

        let operator_id = U256::from_str(
            "35344093966194310405039483339636912150346494903629410125452342281826147822033",
        )
        .unwrap();

        let _ = avs_reader
            .get_operators_stake_in_quorums_at_block_operator_id(1245842, operator_id.into())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_operators_stake_in_quorums_at_current_block() {
        let avs_reader = build_avs_registry_chain_reader().await;
        let quorum_number = Bytes::from_hex("0x00").expect("bytes parse");

        let _ = avs_reader
            .get_operators_stake_in_quorums_at_current_block(quorum_number)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_operators_stake_in_quorums_of_operator_at_block() {
        let avs_reader = build_avs_registry_chain_reader().await;

        let operator_id = U256::from_str(
            "35344093966194310405039483339636912150346494903629410125452342281826147822033",
        )
        .unwrap();

        let _ = avs_reader
            .get_operators_stake_in_quorums_of_operator_at_block((operator_id).into(), 1246078)
            .await
            .unwrap();
    }
}
