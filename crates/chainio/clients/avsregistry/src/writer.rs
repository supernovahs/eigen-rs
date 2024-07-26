use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use ark_bn254::{Fq, G1Affine};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use eigen_chainio_utils::{convert_to_bn254_g1_point, convert_to_bn254_g2_point};
use eigen_client_elcontracts::reader::ELChainReader;
use eigen_utils::binding::{
    BLSApkRegistry::{G1Point, G2Point,PubkeyRegistrationParams},
    RegistryCoordinator::{
        self, G1Point as RegistryG1Point, G2Point as RegistryG2Point,
        PubkeyRegistrationParams as RegistryPubkeyRegistrationParams,
    }
};
use std::str::FromStr;

use alloy_primitives::{Address, Bytes, FixedBytes, TxHash, U256};
use eigen_crypto_bls::{BlsKeypair, EthConvert};
use tracing::info;
use RegistryCoordinator::SignatureWithSaltAndExpiry;

use crate::error::AvsRegistryError;
use eigen_utils::{
    binding::{ServiceManagerBase, StakeRegistry},
    get_provider, get_signer,
};

/// AvsRegistry Writer
#[derive(Debug)]
pub struct AvsRegistryChainWriter {
    service_manager_addr: Address,
    registry_coordinator_addr: Address,
    operator_state_retriever_addr: Address,
    stake_registry_addr: Address,
    bls_apk_registry_addr: Address,
    el_reader: ELChainReader,
    provider: String,
    signer: String,
}

impl AvsRegistryChainWriter {
    /// build avs registry chain writer instance
    pub async fn build_avs_registry_chain_writer(
        provider: String,
        signer: String,
        registry_coordinator_addr: Address,
        operator_state_retriever_addr: Address,
    ) -> Result<Self, AvsRegistryError> {
        let fill_provider = get_provider(&provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(registry_coordinator_addr, &fill_provider);

        let service_manager_addr_result =
            contract_registry_coordinator.serviceManager().call().await;

        match service_manager_addr_result {
            Ok(service_manager_addr) => {
                let RegistryCoordinator::serviceManagerReturn {
                    _0: service_manager,
                } = service_manager_addr;
                let contract_service_manager_base =
                    ServiceManagerBase::new(service_manager, &fill_provider);

                let bls_apk_registry_addr_result =
                    contract_registry_coordinator.blsApkRegistry().call().await;

                match bls_apk_registry_addr_result {
                    Ok(bls_apk_registry_addr) => {
                        let RegistryCoordinator::blsApkRegistryReturn {
                            _0: bls_apk_registry,
                        } = bls_apk_registry_addr;
                        let stake_registry_addr =
                            contract_registry_coordinator.stakeRegistry().call().await?;
                        let RegistryCoordinator::stakeRegistryReturn { _0: stake_registry } =
                            stake_registry_addr;
                        let contract_stake_registry =
                            StakeRegistry::new(stake_registry, &fill_provider);

                        let delegation_manager_return =
                            contract_stake_registry.delegation().call().await?;

                        let StakeRegistry::delegationReturn {
                            _0: delegation_manager_addr,
                        } = delegation_manager_return;
                        let avs_directory_addr =
                            contract_service_manager_base.avsDirectory().call().await?;

                        let ServiceManagerBase::avsDirectoryReturn { _0: avs_directory } =
                            avs_directory_addr;

                        let el_reader_result =
                            ELChainReader::build(delegation_manager_addr, avs_directory, &provider)
                                .await;

                        match el_reader_result {
                            Ok(el_reader) => Ok(AvsRegistryChainWriter {
                                service_manager_addr: service_manager,
                                registry_coordinator_addr,
                                operator_state_retriever_addr,
                                stake_registry_addr: stake_registry,
                                bls_apk_registry_addr: bls_apk_registry,
                                el_reader,
                                provider: provider.clone(),
                                signer: signer.clone(),
                            }),
                            Err(e) => Err(AvsRegistryError::ElContractsError(e.to_string())),
                        }
                    }
                    Err(e) => Err(AvsRegistryError::AlloyContractError(e)),
                }
            }
            Err(e) => Err(AvsRegistryError::AlloyContractError(e)),
        }
    }

    /// Register operator in quorum with avs registry coordinator
    pub async fn register_operator_in_quorum_with_avs_registry_coordinator(
        &self,
        bls_key_pair: BlsKeypair,
        operator_to_avs_registration_sig_salt: FixedBytes<32>,
        operator_to_avs_registration_sig_expiry: U256,
        quorum_numbers: Bytes,
        socket: String,
    ) -> Result<TxHash, AvsRegistryError> {
        let provider = get_signer(self.signer.clone(), &self.provider);
        let wallet = PrivateKeySigner::from_str(&self.signer).expect("failed to generate wallet ");

        // tracing info
        info!(avs_service_manager = %self.service_manager_addr, operator= %wallet.address(),quorum_numbers = ?quorum_numbers,"quorum_numbers,registering operator with the AVS's registry coordinator");
        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);
        let g1_hashes_msg_to_sign_return_result = contract_registry_coordinator
            .pubkeyRegistrationMessageHash(wallet.address())
            .call()
            .await?;
        let RegistryCoordinator::pubkeyRegistrationMessageHashReturn {
            _0: g1_hashes_msg_to_sign,
        } = g1_hashes_msg_to_sign_return_result;
      
        let mut serialized_bytes = vec![];

        println!("g1 hashes msg to sign : {}", g1_hashes_msg_to_sign.X);
        println!("g1 hashes msg to sign : {}", g1_hashes_msg_to_sign.Y);

        let g1_affine = G1Affine::new(
            Fq::from_str(&g1_hashes_msg_to_sign.X.to_string()).unwrap(),
            Fq::from_str(&g1_hashes_msg_to_sign.Y.to_string()).unwrap(),
        );
        g1_affine
            .serialize_uncompressed(&mut serialized_bytes)
            .unwrap();

        let signature = bls_key_pair.sign(&serialized_bytes).unwrap();

        // let verified = BlsKeypair::verify_g1_public(bls_key_pair.public,signature,g1_affine);

        let g1_pubkey_bn254:Option<G1Point> = bls_key_pair.clone().into();

        let g2_pubkey_bn254 :Option<G2Point> = bls_key_pair.into();

        let pub_key_reg_params = PubkeyRegistrationParams {
            pubkeyRegistrationSignature: EthConvert::to_g1(signature).unwrap(),
            pubkeyG1: g1_pubkey_bn254.expect("g1 is none"),
            pubkeyG2: g2_pubkey_bn254.expect("g2 is none"),
        };

        
                let msg_to_sign_result = self
                    .el_reader
                    .calculate_operator_avs_registration_digest_hash(
                        wallet.address(),
                        self.service_manager_addr,
                        operator_to_avs_registration_sig_salt,
                        operator_to_avs_registration_sig_expiry,
                    )
                    .await;

                match msg_to_sign_result {
                    Ok(msg_to_sign) => {
                        let operator_signature = wallet
                            .sign_message_sync(msg_to_sign.as_slice())
                            .expect("failed to sign message");

                        let operator_signature_with_salt_and_expiry = SignatureWithSaltAndExpiry {
                            signature: operator_signature.as_bytes().into(),
                            salt: operator_to_avs_registration_sig_salt,
                            expiry: operator_to_avs_registration_sig_expiry,
                        };

                        let contract_call = contract_registry_coordinator.registerOperator(
                            quorum_numbers.clone(),
                            socket,
                            RegistryPubkeyRegistrationParams {
                                pubkeyRegistrationSignature: RegistryG1Point {
                                    X: pub_key_reg_params.pubkeyRegistrationSignature.X,
                                    Y: pub_key_reg_params.pubkeyRegistrationSignature.Y,
                                },
                                pubkeyG1: RegistryG1Point {
                                    X: pub_key_reg_params.pubkeyG1.X,
                                    Y: pub_key_reg_params.pubkeyG1.Y,
                                },
                                pubkeyG2: RegistryG2Point {
                                    X: pub_key_reg_params.pubkeyG2.X,
                                    Y: pub_key_reg_params.pubkeyG2.Y,
                                },
                            },
                            operator_signature_with_salt_and_expiry,
                        );

                        let tx_call = contract_call.gas(2000000);
                        let tx_result = tx_call.send().await;

                        match tx_result {
                            Ok(tx) => {
                                info!(tx_hash = ?tx,"succesfully deregistered operator with the AVS's registry coordinator" );
                                return Ok(*tx.tx_hash());
                            }
                            Err(e) => Err(AvsRegistryError::AlloyContractError(e)),
                        }
                    }
                    Err(e) => Err(AvsRegistryError::ElContractsError(e.to_string())),
                }
            }

    /// Update stakes of entire operator set for quorums
    pub async fn update_stakes_of_entire_operator_set_for_quorums(
        &self,
        operators_per_quorum: Vec<Vec<Address>>,
        quorum_number: Bytes,
    ) -> Result<TxHash, AvsRegistryError> {
        info!(quorum_numbers = %quorum_number, "updating stakes for entire operator set");
        let provider = get_signer(self.signer.clone(), &self.provider);
        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let contract_call = contract_registry_coordinator
            .updateOperatorsForQuorum(operators_per_quorum, quorum_number.clone());

        let tx_result = contract_call.send().await;

        match tx_result {
            Ok(tx) => {
                info!(tx_hash = ?tx, quorum_numbers = %quorum_number," update stakes for entire operator set tx" );
                return Ok(*tx.tx_hash());
            }
            Err(e) => Err(AvsRegistryError::AlloyContractError(e)),
        }
    }

    /// Update stakes of operator subset for all quorums
    pub async fn update_stakes_of_operator_subset_for_all_quorums(
        &self,
        operators: Vec<Address>,
    ) -> Result<TxHash, AvsRegistryError> {
        info!(operators = ?operators, "updating stakes of operator subset for all quorums");

        let provider = get_signer(self.signer.clone(), &self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let contract_call = contract_registry_coordinator.updateOperators(operators);

        let tx_result = contract_call.send().await;

        match tx_result {
            Ok(tx) => {
                info!(tx_hash = ?tx,"succesfully updated stakes of operator subset for all quorums" );
                Ok(*tx.tx_hash())
            }
            Err(e) => Err(AvsRegistryError::AlloyContractError(e)),
        }
    }

    /// Deregister operator
    pub async fn deregister_operator(
        &self,
        quorum_numbers: Bytes,
    ) -> Result<TxHash, AvsRegistryError> {
        info!("deregistering operator with the AVS's registry coordinator");
        let provider = get_signer(self.signer.clone(), &self.provider);

        let contract_registry_coordinator =
            RegistryCoordinator::new(self.registry_coordinator_addr, provider);

        let contract_call = contract_registry_coordinator.deregisterOperator(quorum_numbers);

        let tx_result = contract_call.send().await;
        match tx_result {
            Ok(tx) => {
                info!(tx_hash = ?tx,"succesfully deregistered operator with the AVS's registry coordinator" );
                Ok(*tx.tx_hash())
            }
            Err(e) => Err(AvsRegistryError::AlloyContractError(e)),
        }
    }
}
