use crate::error::ElContractsError;
use crate::reader::ELChainReader;
use alloy_primitives::FixedBytes;
use alloy_primitives::{Address, TxHash, U256};
pub use eigen_types::operator::Operator;
use eigen_utils::{
    binding::{
        DelegationManager::{self},
        StrategyManager, IERC20,
    },
    get_signer,
};

use tracing::info;
use DelegationManager::OperatorDetails;

#[derive(Debug, Clone)]
pub struct ELChainWriter {
    delegation_manager: Address,
    strategy_manager: Address,
    el_chain_reader: ELChainReader,
    provider: String,
    signer: String,
}

impl ELChainWriter {
    pub fn new(
        delegation_manager: Address,
        strategy_manager: Address,
        el_chain_reader: ELChainReader,
        provider: String,
        signer: String,
    ) -> Self {
        Self {
            delegation_manager,
            strategy_manager,
            el_chain_reader,
            provider,
            signer,
        }
    }

    pub async fn register_as_operator(
        &self,
        operator: Operator,
    ) -> Result<FixedBytes<32>, ElContractsError> {
        info!(
            "registering operator {:?} to EigenLayer",
            operator.has_address()
        );
        let op_details = OperatorDetails {
            earningsReceiver: operator.has_earnings_receiver_address(),
            delegationApprover: operator.has_delegation_approver_address(),
            stakerOptOutWindowBlocks: operator.has_staker_opt_out_window_blocks(),
        };
        let provider = get_signer(self.signer.clone(), &self.provider);

        let contract_delegation_manager = DelegationManager::new(self.delegation_manager, provider);

        let binding = match operator.has_metadata_url() {
            Some(metadata) => {
                let contract_call =
                    contract_delegation_manager.registerAsOperator(op_details, metadata);
                contract_call.gas(300000)
            }
            None => {
                let contract_call =
                    contract_delegation_manager.registerAsOperator(op_details, "".to_string());
                contract_call.gas(300000)
            }
        };
        let binding_tx_result = binding.send().await;
        match binding_tx_result {
            Ok(binding_tx) => {
                let receipt_result = binding_tx.get_receipt().await;
                match receipt_result {
                    Ok(receipt) => {
                        let tx_status = receipt.status();
                        let hash = receipt.transaction_hash;
                        match tx_status {
                            true => {
                                info!(tx_hash = %receipt.transaction_hash, "tx successfully included");
                                Ok(hash)
                            }
                            false => {
                                info!(tx_hash = %receipt.transaction_hash, "failed to register operator");
                                Ok(hash)
                            }
                        }
                    }
                    Err(e) => Err(ElContractsError::AlloyContractError(
                        alloy_contract::Error::TransportError(e),
                    )),
                }
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn update_operator_details(
        &self,
        operator: Operator,
    ) -> Result<TxHash, ElContractsError> {
        info!(
            "updating operator detils of operator {:?} to EigenLayer",
            operator.has_address()
        );
        let operator_details = OperatorDetails {
            earningsReceiver: operator.has_earnings_receiver_address(),
            delegationApprover: operator.has_delegation_approver_address(),
            stakerOptOutWindowBlocks: operator.has_staker_opt_out_window_blocks(),
        };
        let provider = get_signer(self.signer.clone(), &self.provider);

        let contract_delegation_manager = DelegationManager::new(self.delegation_manager, provider);

        let contract_call_modify_operator_details =
            contract_delegation_manager.modifyOperatorDetails(operator_details);

        let modify_operator_tx_result = contract_call_modify_operator_details.send().await;

        match modify_operator_tx_result {
            Ok(modify_operator_tx) => {
                info!(tx_hash = %modify_operator_tx.tx_hash(), operator = %operator.has_address(), "updated operator details tx");

                let contract_call_update_metadata_uri = contract_delegation_manager
                    .updateOperatorMetadataURI(operator.has_metadata_url().unwrap_or_default());

                let metadata_tx = contract_call_update_metadata_uri.send().await?;

                Ok(*metadata_tx.tx_hash())
            }
            Err(e) => Err(ElContractsError::AlloyContractError(e)),
        }
    }

    pub async fn deposit_erc20_into_strategy(
        &self,
        strategy_addr: Address,
        amount: U256,
    ) -> Result<TxHash, ElContractsError> {
        info!(
            "depositing {:?} tokens into strategy {:?}",
            amount, strategy_addr
        );
        let tokens_result = self
            .el_chain_reader
            .get_strategy_and_underlying_erc20_token(strategy_addr)
            .await;

        match tokens_result {
            Ok(tokens) => {
                let (_, underlying_token_contract, underlying_token) = tokens;
                let provider = get_signer(self.signer.clone(), &self.provider);

                let contract_underlying_token = IERC20::new(underlying_token_contract, &provider);

                let contract_call =
                    contract_underlying_token.approve(self.strategy_manager, amount);

                let _approve = contract_call.send().await?;

                let contract_strategy_manager =
                    StrategyManager::new(self.strategy_manager, &provider);

                let deposit_contract_call = contract_strategy_manager.depositIntoStrategy(
                    strategy_addr,
                    underlying_token,
                    amount,
                );

                let tx = deposit_contract_call.send().await?;

                info!(
                    "deposited {:?} tokens into strategy {:?}",
                    amount, strategy_addr
                );
                Ok(*tx.tx_hash())
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloy_signer_local::PrivateKeySigner;
    use eigen_logging::get_test_logger;
    use eigen_testing_utils::anvil_constants::{self};
    use eigen_utils::binding::{
        mockAvsServiceManager,
        ContractsRegistry::{self, get_test_valuesReturn},
    };

    #[tokio::test]
    async fn test_register_operator() {
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
        let operator_pvt_key = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
        let operator: PrivateKeySigner = (operator_pvt_key)
            .parse()
            .expect("failed to generate wallet");
        let avs_directory_address_return = service_manager_contract
            .avsDirectory()
            .call()
            .await
            .unwrap();
        let mockAvsServiceManager::avsDirectoryReturn {
            _0: avs_directory_address,
        } = avs_directory_address_return;

        let el_chain_reader = ELChainReader::new(
            get_test_logger().clone(),
            slasher_address,
            delegation_manager_address,
            avs_directory_address,
            "http://localhost:8545".to_string(),
        );
        let contract_registry = ContractsRegistry::new(
            anvil_constants::CONTRACTS_REGISTRY,
            anvil_constants::ANVIL_RPC_URL.clone(),
        );
        // Use these value in tests when needed
        let operator_index = "1".parse().unwrap();
        let get_test_values_return = contract_registry
            .get_test_values("test_register_operator".to_string(), operator_index)
            .call()
            .await
            .unwrap();
        let get_test_valuesReturn {
            _0: _timestamp,
            _1: _blocknumber,
            _2: _index,
        } = get_test_values_return;

        // operator who registered at index 1
        let operator_address = operator.address();
        assert!(el_chain_reader
            .is_operator_registered(operator_address)
            .await
            .unwrap());
    }
}
