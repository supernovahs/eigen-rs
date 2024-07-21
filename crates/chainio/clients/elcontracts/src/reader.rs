use crate::error::ElContractsError;
use alloy_primitives::{Address, FixedBytes, U256};
use eigen_types::operator::Operator;
use eigen_utils::{
    binding::{AVSDirectory, DelegationManager, ISlasher, IStrategy, IERC20},
    get_provider,
};

#[derive(Debug, Clone)]
pub struct ELChainReader {
    slasher: Address,
    delegation_manager: Address,
    avs_directory: Address,
    provider: String,
}

impl ELChainReader {
    pub fn new(
        slasher: Address,
        delegation_manager: Address,
        avs_directory: Address,
        provider: String,
    ) -> Self {
        ELChainReader {
            slasher,
            delegation_manager,
            avs_directory,
            provider,
        }
    }

    pub async fn build(
        delegation_manager: Address,
        avs_directory: Address,
        client: &String,
    ) -> Result<Self, ElContractsError> {
        let provider = get_provider(client);

        let contract_delegation_manager = DelegationManager::new(delegation_manager, provider);

        let slasher_result = contract_delegation_manager.slasher().call().await;

        match slasher_result {
            Ok(slasher) => {
                let DelegationManager::slasherReturn { _0: slasher_addr } = slasher;

                Ok(Self {
                    avs_directory,
                    slasher: slasher_addr,
                    delegation_manager,
                    provider: client.to_string(),
                })
            }

            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn calculate_delegation_approval_digest_hash(
        &self,
        staker: Address,
        operator: Address,
        delegation_approver: Address,
        approve_salt: FixedBytes<32>,
        expiry: U256,
    ) -> Result<FixedBytes<32>, ElContractsError> {
        let provider = get_provider(&self.provider);
        let contract_delegation_manager = DelegationManager::new(self.delegation_manager, provider);
        let delegation_approval_digest_hash_result = contract_delegation_manager
            .calculateDelegationApprovalDigestHash(
                staker,
                operator,
                delegation_approver,
                approve_salt,
                expiry,
            )
            .call()
            .await;
        match delegation_approval_digest_hash_result {
            Ok(delegation_approval_digest_hash) => {
                let DelegationManager::calculateDelegationApprovalDigestHashReturn {
                    _0: digest_hash,
                } = delegation_approval_digest_hash;

                Ok(digest_hash)
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn calculate_operator_avs_registration_digest_hash(
        &self,
        operator: Address,
        avs: Address,
        salt: FixedBytes<32>,
        expiry: U256,
    ) -> Result<FixedBytes<32>, ElContractsError> {
        let provider = get_provider(&self.provider);

        let contract_avs_directory = AVSDirectory::new(self.avs_directory, provider);

        let operator_avs_registration_digest_hash_result = contract_avs_directory
            .calculateOperatorAVSRegistrationDigestHash(operator, avs, salt, expiry)
            .call()
            .await;

        match operator_avs_registration_digest_hash_result {
            Ok(operator_avs_registration_digest_hash) => {
                let AVSDirectory::calculateOperatorAVSRegistrationDigestHashReturn { _0: avs_hash } =
                    operator_avs_registration_digest_hash;

                Ok(avs_hash)
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn get_operator_shares_in_strategy(
        &self,
        operator_addr: Address,
        strategy_addr: Address,
    ) -> Result<U256, ElContractsError> {
        let provider = get_provider(&self.provider);

        let contract_delegation_manager = DelegationManager::new(self.delegation_manager, provider);

        let operator_shares_in_strategy_result = contract_delegation_manager
            .operatorShares(operator_addr, strategy_addr)
            .call()
            .await;

        match operator_shares_in_strategy_result {
            Ok(operator_shares_in_strategy) => {
                let DelegationManager::operatorSharesReturn { _0: shares } =
                    operator_shares_in_strategy;
                Ok(shares)
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn operator_is_frozen(
        &self,
        operator_addr: Address,
    ) -> Result<bool, ElContractsError> {
        let provider = get_provider(&self.provider);

        let contract_slasher = ISlasher::new(self.slasher, provider);

        let operator_is_frozen_result = contract_slasher.isFrozen(operator_addr).call().await;

        match operator_is_frozen_result {
            Ok(operator_is_frozen) => {
                let ISlasher::isFrozenReturn { _0: is_froze } = operator_is_frozen;
                Ok(is_froze)
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn service_manager_can_slash_operator_until_block(
        &self,
        operator_addr: Address,
        service_manager_addr: Address,
    ) -> Result<u32, ElContractsError> {
        let provider = get_provider(&self.provider);

        let contract_slasher = ISlasher::new(self.slasher, provider);

        let service_manager_can_slash_operator_until_block_result = contract_slasher
            .contractCanSlashOperatorUntilBlock(operator_addr, service_manager_addr)
            .call()
            .await;

        match service_manager_can_slash_operator_until_block_result {
            Ok(service_manager_can_slash_operator_until_block) => {
                let ISlasher::contractCanSlashOperatorUntilBlockReturn { _0: can_slash } =
                    service_manager_can_slash_operator_until_block;

                Ok(can_slash)
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn get_strategy_and_underlying_erc20_token(
        &self,
        strategy_addr: Address,
    ) -> Result<(Address, Address, Address), ElContractsError> {
        let provider = get_provider(&self.provider);

        let contract_strategy = IStrategy::new(strategy_addr, &provider);

        let underlying_token_result = contract_strategy.underlyingToken().call().await;

        match underlying_token_result {
            Ok(underlying_token) => {
                let IStrategy::underlyingTokenReturn {
                    _0: underlying_token_addr,
                } = underlying_token;

                let contract_ierc20 = IERC20::new(underlying_token_addr, &provider);

                return Ok((
                    strategy_addr,
                    underlying_token_addr,
                    *contract_ierc20.address(),
                ));
            }

            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn get_operator_details(
        &self,
        operator: Address,
    ) -> Result<Operator, ElContractsError> {
        let provider = get_provider(&self.provider);

        let contract_delegation_manager =
            DelegationManager::new(self.delegation_manager, &provider);

        let operator_det_result = contract_delegation_manager
            .operatorDetails(operator)
            .call()
            .await;

        match operator_det_result {
            Ok(operator_det) => {
                let DelegationManager::operatorDetailsReturn {
                    _0: operator_details,
                } = operator_det;

                Ok(Operator::new(
                    operator,
                    operator_details.earningsReceiver,
                    operator_details.delegationApprover,
                    operator_details.stakerOptOutWindowBlocks,
                    None,
                ))
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn is_operator_registered(
        &self,
        operator: Address,
    ) -> Result<bool, ElContractsError> {
        let provider = get_provider(&self.provider);

        let contract_delegation_manager = DelegationManager::new(self.delegation_manager, provider);

        let is_operator_result = contract_delegation_manager
            .isOperator(operator)
            .call()
            .await;

        match is_operator_result {
            Ok(is_operator) => {
                let DelegationManager::isOperatorReturn { _0: is_operator_is } = is_operator;
                Ok(is_operator_is)
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }
}

/// Anvil tests
#[cfg(test)]
mod tests {

    use super::*;
    use alloy_eips::eip1898::BlockNumberOrTag::Number;
    use alloy_primitives::{address, keccak256};
    use alloy_provider::Provider;
    use eigen_testing_utils::anvil_constants::{self, ANVIL_RPC_URL};
    use eigen_utils::binding::mockAvsServiceManager;
    use tokio::time::{sleep, Duration};
    use AVSDirectory::calculateOperatorAVSRegistrationDigestHashReturn;
    use DelegationManager::calculateDelegationApprovalDigestHashReturn;

    async fn build_el_chain_reader() -> ELChainReader {
        let delegation_manager_address = anvil_constants::get_delegation_manager_address().await;
        let delegation_manager_contract = DelegationManager::new(
            delegation_manager_address,
            anvil_constants::ANVIL_RPC_URL.clone(),
        );
        let slasher_address_return = delegation_manager_contract.slasher().call().await.unwrap();
        let DelegationManager::slasherReturn {
            _0: slasher_address,
        } = slasher_address_return;
        let service_manager_address = anvil_constants::get_service_manager_address().await;
        let service_manager_contract = mockAvsServiceManager::new(
            service_manager_address,
            anvil_constants::ANVIL_RPC_URL.clone(),
        );
        let avs_directory_address_return = service_manager_contract
            .avsDirectory()
            .call()
            .await
            .unwrap();
        let mockAvsServiceManager::avsDirectoryReturn {
            _0: avs_directory_address,
        } = avs_directory_address_return;
        ELChainReader::new(
            slasher_address,
            delegation_manager_address,
            avs_directory_address,
            "http://localhost:8545".to_string(),
        )
    }

    #[tokio::test]
    async fn test_calculate_delegation_approval_digest_hash() {
        // Introduce a 2-second delay
        sleep(Duration::from_secs(2)).await;
        let el_chain_reader = build_el_chain_reader().await;
        let operator: Address = address!("5eb15C0992734B5e77c888D713b4FC67b3D679A2");

        let staker = operator;

        let delegation_approver = Address::ZERO;

        let approve_salt: FixedBytes<32> = FixedBytes::from([
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02,
        ]);
        let current_block_number = ANVIL_RPC_URL.clone().get_block_number().await.unwrap();
        let block_info = ANVIL_RPC_URL
            .clone()
            .get_block_by_number(Number(current_block_number), true)
            .await
            .unwrap();

        if let Some(block) = block_info {
            let timestamp = block.header.timestamp;
            let expiry = U256::from::<u64>(timestamp + 100);
            let calculate_digest_hash = el_chain_reader
                .calculate_delegation_approval_digest_hash(
                    staker,
                    operator,
                    delegation_approver,
                    approve_salt,
                    expiry,
                )
                .await
                .unwrap();

            // Directly calling the function through bindings to compare with the sdk .
            let delegation_manager_address =
                anvil_constants::get_delegation_manager_address().await;
            let delegation_manager_contract = DelegationManager::new(
                delegation_manager_address,
                anvil_constants::ANVIL_RPC_URL.clone(),
            );

            let hash = delegation_manager_contract
                .calculateDelegationApprovalDigestHash(
                    staker,
                    operator,
                    delegation_approver,
                    approve_salt,
                    expiry,
                )
                .call()
                .await
                .unwrap();

            let calculateDelegationApprovalDigestHashReturn { _0: digest_hash } = hash;

            assert_eq!(digest_hash, calculate_digest_hash);
        }
    }

    #[tokio::test]
    async fn test_calculate_operator_avs_registration_digest_hash() {
        // a 2 se
        sleep(Duration::from_secs(2)).await;
        let el_chain_reader = build_el_chain_reader().await;
        let operator: Address = address!("5eb15C0992734B5e77c888D713b4FC67b3D679A2");
        let avs = Address::from_slice(&keccak256("avs ")[0..20]);
        let salt: FixedBytes<32> = FixedBytes::from([
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02,
        ]);
        let current_block_number = ANVIL_RPC_URL.clone().get_block_number().await.unwrap();
        let block_info = ANVIL_RPC_URL
            .clone()
            .get_block_by_number(Number(current_block_number), true)
            .await
            .unwrap();
        if let Some(block) = block_info {
            let timestamp = block.header.timestamp;
            let expiry = U256::from::<u64>(timestamp + 100);
            let operator_hash = el_chain_reader
                .calculate_operator_avs_registration_digest_hash(operator, avs, salt, expiry)
                .await
                .unwrap();

            // Using bindings directly to compare with sdk's output
            let avs_registry_contract =
                AVSDirectory::new(el_chain_reader.avs_directory, ANVIL_RPC_URL.clone());
            let operator_hash_from_bindings = avs_registry_contract
                .calculateOperatorAVSRegistrationDigestHash(operator, avs, salt, expiry)
                .call()
                .await
                .unwrap();

            let calculateOperatorAVSRegistrationDigestHashReturn { _0: hash } =
                operator_hash_from_bindings;

            assert_eq!(hash, operator_hash);
        }
    }
}
