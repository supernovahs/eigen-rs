// use alloy_primitives::{Bytes, FixedBytes, U256};
// use eigen_client_avsregistry::reader::AvsRegistryChainReader;
// use eigen_crypto_bls::BlsG1Point;
// use eigen_services_operatorsinfo::operatorsinfo_inmemory::OperatorInfoServiceInMemory;
// use eigen_types::operator::{OperatorAvsState, OperatorInfo, OperatorPubKeys, QuorumAvsState};
// use eigen_utils::binding::BLSApkRegistry::G1Point;
// use std::collections::HashMap;

// #[derive(Debug)]
// pub struct AvsRegistryServiceChainCaller {
//     avs_registry: AvsRegistryChainReader,
//     operators_info_service: OperatorInfoServiceInMemory,
// }

// impl AvsRegistryServiceChainCaller {
//     pub fn new(
//         avs_registry: AvsRegistryChainReader,
//         operators_info_service: OperatorInfoServiceInMemory,
//     ) -> Self {
//         Self {
//             avs_registry,
//             operators_info_service,
//         }
//     }

//     pub fn get_avs_registry(&self) -> AvsRegistryChainReader {
//         self.avs_registry.clone()
//     }

//     pub async fn get_operators_avs_state_at_block(
//         &self,
//         block_num: u32,
//         quorum_nums: Bytes,
//     ) -> HashMap<FixedBytes<32>, OperatorAvsState> {
//         let mut operators_avs_state: HashMap<FixedBytes<32>, OperatorAvsState> = HashMap::new();

//         let operators_stakes_in_quorums = self
//             .avs_registry
//             .get_operators_stake_in_quorums_at_block(block_num, quorum_nums.clone())
//             .await
//             .unwrap();

//         if operators_stakes_in_quorums.len() != quorum_nums.len() {
//             // throw error
//         }

//         for (quorum_id, quorum_num) in quorum_nums.iter().enumerate() {
//             for operator in &operators_stakes_in_quorums[quorum_id] {
//                 let info = self.get_operator_info(*operator.operatorId).await;
//                 let stake_per_quorum = HashMap::new();
//                 let avs_state = operators_avs_state
//                     .entry(FixedBytes(*operator.operatorId))
//                     .or_insert_with(|| OperatorAvsState {
//                         operator_id: *operator.operatorId,
//                         operator_info: OperatorInfo { pub_keys: info },
//                         stake_per_quorum,
//                         block_num: block_num.into(),
//                     });
//                 avs_state
//                     .stake_per_quorum
//                     .insert(*quorum_num, U256::from(operator.stake));
//             }
//         }

//         operators_avs_state
//     }

//     pub async fn get_quorums_avs_state_at_block(
//         &self,
//         quorum_nums: Bytes,
//         block_num: u32,
//     ) -> HashMap<u8, QuorumAvsState> {
//         let operators_avs_state = self
//             .get_operators_avs_state_at_block(block_num, quorum_nums.clone())
//             .await;

//         let mut quorums_avs_state: HashMap<u8, QuorumAvsState> = HashMap::new();

//         for quorum_num in quorum_nums.iter() {
//             let mut pub_key_g1 = BlsG1Point::new(
//                 u256_to_bigint256(U256::from(0)),
//                 u256_to_bigint256(U256::from(0)),
//             );
//             let mut total_stake: U256 = U256::from(0);
//             for operator in operators_avs_state.values() {
//                 if !operator.stake_per_quorum[quorum_num].is_zero() {
//                     if let Some(pubkeys) = &operator.operator_info.pub_keys {
//                         let g1_point = BlsG1Point::new(
//                             u256_to_bigint256(pubkeys.g1_pub_key.X),
//                             u256_to_bigint256(pubkeys.g1_pub_key.Y),
//                         );
//                         pub_key_g1.add(g1_point);
//                         total_stake += operator.stake_per_quorum[quorum_num];
//                     }
//                 }
//             }
//             let g1_point = convert_to_bn254_g1_point(pub_key_g1.point);
//             quorums_avs_state.insert(
//                 *quorum_num,
//                 QuorumAvsState {
//                     quorum_num: *quorum_num,
//                     total_stake,
//                     agg_pub_key_g1: G1Point {
//                         X: g1_point.X,
//                         Y: g1_point.Y,
//                     },
//                     block_num,
//                 },
//             );
//         }
//         quorums_avs_state
//     }

//     pub async fn get_operator_info(&self, operator_id: [u8; 32]) -> Option<OperatorPubKeys> {
//         let operator_addr = self
//             .avs_registry
//             .get_operator_from_id(operator_id)
//             .await
//             .unwrap();

//         self.operators_info_service
//             .get_operator_info(operator_addr)
//             .await
//     }
// }
