// use eigen_crypto_bls::attestation::{G1Point, G2Point, Signature};
// use eigen_services_avsregistry::chaincaller::AvsRegistryServiceChainCaller;
// use eigen_types::{
//     avs::{SignedTaskResponseDigest, TaskIndex, TaskResponseDigest},
//     operator::{OperatorAvsState, QuorumThresholdPercentage, QuorumThresholdPercentages},
// };

// use alloy_primitives::{FixedBytes, U256};
// use std::collections::HashMap;
// use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
// use tokio::time::{self, Duration};

// use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
// use std::sync::Arc;

// #[allow(unused)]
// #[derive(Debug, Clone)]
// pub struct BlsAggregationServiceResponse {
//     task_index: TaskIndex,
//     task_response_digest: TaskResponseDigest,
//     non_signers_pub_keys_g1: Vec<G1Point>,
//     quorum_apks_g1: Vec<G1Point>,
//     signers_apk_g2: G2Point,
//     signers_agg_sig_g1: Signature,
//     non_signer_quorum_bitmap_indices: Vec<u32>,
//     quorum_apk_indices: Vec<u32>,
//     total_stake_indices: Vec<u32>,
//     non_signer_stake_indices: Vec<Vec<u32>>,
// }

// #[derive(Debug, Clone)]
// pub struct AggregatedOperators {
//     signers_apk_g2: G2Point,

//     signers_agg_sig_g1: Signature,

//     signers_total_stake_per_quorum: HashMap<u8, U256>,

//     pub signers_operator_ids_set: HashMap<FixedBytes<32>, bool>,
// }

// impl AggregatedOperators {}
// #[derive(Debug)]
// pub struct BlsAggregatorService {
//     aggregated_response_sender: UnboundedSender<BlsAggregationServiceResponse>,
//     pub aggregated_response_receiver: UnboundedReceiver<BlsAggregationServiceResponse>,
//     signed_task_response:
//         Arc<RwLock<HashMap<TaskIndex, UnboundedSender<SignedTaskResponseDigest>>>>,
//     avs_registry_service: AvsRegistryServiceChainCaller,
// }

// impl BlsAggregatorService {
//     pub fn new(avs_registry_service: AvsRegistryServiceChainCaller) -> Self {
//         let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//         Self {
//             aggregated_response_sender: tx,
//             aggregated_response_receiver: rx,
//             signed_task_response: Arc::new(RwLock::new(HashMap::new())),
//             avs_registry_service,
//         }
//     }

//     pub(crate) async fn write(
//         &self,
//     ) -> RwLockWriteGuard<'_, HashMap<TaskIndex, UnboundedSender<SignedTaskResponseDigest>>> {
//         self.signed_task_response.write()
//     }

//     pub(crate) async fn read(
//         &self,
//     ) -> RwLockReadGuard<'_, HashMap<TaskIndex, UnboundedSender<SignedTaskResponseDigest>>> {
//         self.signed_task_response.read()
//     }

//     pub async fn initialize_new_task(
//         self: Arc<Self>,
//         task_index: TaskIndex,
//         task_created_block: u32,
//         quorum_nums: Vec<u8>,
//         quorum_threshold_percentages: QuorumThresholdPercentages,
//         time_to_expiry: Duration,
//     ) {
//         let mut task_channel = self.write().await;

//         if task_channel.contains_key(&task_index) {
//             // error
//         }

//         let (tx, mut rx) = mpsc::unbounded_channel();
//         task_channel.insert(task_index, tx);
//         let self_clone = self.clone();
//         tokio::spawn(async move {
//             while let Some(signed_response) = rx.recv().await {
//                 // Process each signed response here
//                 self_clone
//                     .single_task_aggregator(
//                         task_index,
//                         task_created_block,
//                         quorum_nums.clone(),
//                         quorum_threshold_percentages.clone(),
//                         time_to_expiry,
//                         signed_response,
//                     )
//                     .await;
//             }
//         });
//     }

//     /// Processs signatures
//     pub async fn process_new_signature(
//         &self,
//         task_index: TaskIndex,
//         task_response_digest: TaskResponseDigest,
//         bls_signature: Signature,
//         operator_id: FixedBytes<32>,
//     ) {
//         let task_channel = self.read().await;

//         if let Some(sender) = task_channel.get(&task_index) {
//             let task = SignedTaskResponseDigest {
//                 task_response_digest,
//                 bls_signature,
//                 operator_id,
//             };
//             let _ = sender.send(task);
//         }
//     }

//     pub async fn single_task_aggregator(
//         &self,
//         task_index: TaskIndex,
//         task_created_block: u32,
//         quorum_nums: Vec<u8>,
//         quorum_threshold_percentages: QuorumThresholdPercentages,
//         time_to_expiry: Duration,
//         signed_task_digest: SignedTaskResponseDigest,
//     ) {
//         let mut quorum_threshold_percentage_map = HashMap::new();

//         for (i, quorum_number) in quorum_nums.iter().enumerate() {
//             quorum_threshold_percentage_map.insert(*quorum_number, quorum_threshold_percentages[i]);
//         }

//         let mut operator_state_avs = self
//             .avs_registry_service
//             .get_operators_avs_state_at_block(task_created_block, quorum_nums.clone().into())
//             .await;
//         // throw erro if

//         let quorums_avs_stake = self
//             .avs_registry_service
//             .get_quorums_avs_state_at_block(quorum_nums.clone().into(), task_created_block)
//             .await;
//         // throw erro if != nil

//         let mut total_stake_per_quorum = HashMap::new();

//         for (quorum_num, quorum_avs_stake) in &quorums_avs_stake {
//             total_stake_per_quorum.insert(*quorum_num, quorum_avs_stake.total_stake);
//         }
//         let mut quorum_apks_g1: Vec<G1Point> = vec![];
//         // let quorum_apks_g1
//         for quorum_number in quorum_nums.iter() {
//             if let Some(val) = quorums_avs_stake.get(quorum_number) {
//                 quorum_apks_g1.push(G1Point::new(
//                     u256_to_bigint256(val.agg_pub_key_g1.X),
//                     u256_to_bigint256(val.agg_pub_key_g1.Y),
//                 ));
//             }
//         }

//         let task_expired_timer = time::sleep(time_to_expiry);

//         tokio::pin!(task_expired_timer);
//         let mut aggregated_operators: HashMap<FixedBytes<32>, AggregatedOperators> = HashMap::new();
//         let _sig = self
//             .verify_signature(task_index, &signed_task_digest, &operator_state_avs)
//             .await;
//         loop {
//             tokio::select! {
//                 _ = &mut task_expired_timer =>{
//                     // todo tracing
//                     println!("time expired");
//                     break;
//                 },
//                 else =>{
//                     let mut aggregate_response: AggregatedOperators;
//                     if let Some(response) = aggregated_operators.get(&signed_task_digest.task_response_digest) {
//                         aggregate_response = response.clone();
//                         if aggregated_operators.contains_key(&signed_task_digest.task_response_digest){
//                             aggregate_response.signers_agg_sig_g1.get_g1_point().add(signed_task_digest.bls_signature.get_g1_point());
//                             if let Some(op_avs_state) = operator_state_avs.get_mut(&signed_task_digest.operator_id){
//                                 if let Some(pub_key) =&op_avs_state.operator_info.pub_keys {
//                                     let g2_pub_key  = G2Point::new((u256_to_bigint256(pub_key.g2_pub_key.X[0]),u256_to_bigint256(pub_key.g2_pub_key.X[1])),(u256_to_bigint256(pub_key.g2_pub_key.Y[0]),u256_to_bigint256(pub_key.g2_pub_key.Y[1])));
//                                     aggregate_response.signers_apk_g2.add(g2_pub_key);

//                                 }
//                             }

//                             aggregate_response.signers_operator_ids_set.insert(FixedBytes(*signed_task_digest.operator_id), true);

//                             if let Some(state_avs) = operator_state_avs.get(&signed_task_digest.operator_id){

//                                 for (quorum_num,stake) in state_avs.stake_per_quorum.clone(){

//                                     if let Some(_quorum) =  aggregate_response.signers_total_stake_per_quorum.get(&quorum_num){
//                                             aggregate_response.signers_total_stake_per_quorum.insert(quorum_num, U256::from(0));

//                                     }
//                                    aggregate_response.signers_total_stake_per_quorum.insert(quorum_num,aggregate_response.signers_total_stake_per_quorum[&quorum_num] + stake);

//                                 }
//                             }

//                         }
//                         else{
//                             let mut operator_id_set = HashMap::new();
//                             operator_id_set.insert(signed_task_digest.operator_id,true);
//                             // first operator

//                             if let Some(avs_state) = operator_state_avs.get(&signed_task_digest.operator_id.clone()){

//                                 aggregate_response = AggregatedOperators{
//                                     signers_agg_sig_g1: signed_task_digest.bls_signature.clone(),
//                                     signers_apk_g2 : G2Point::new_zero_g2_point(),
//                                     signers_operator_ids_set: operator_id_set,
//                                     signers_total_stake_per_quorum: avs_state.stake_per_quorum.clone()
//                                 } ;
//                             }
//                         }

//                         aggregated_operators.insert(signed_task_digest.task_response_digest, aggregate_response.clone());
//                         // check stake threshold

//                         if self.check_if_stake_thresholds_met(aggregate_response.signers_total_stake_per_quorum,total_stake_per_quorum.clone(),quorum_threshold_percentage_map.clone()){
//                             let mut non_signers_operators_ids: Vec<FixedBytes<32>> = vec![];
//                             for (i,op_info) in &operator_state_avs{
//                                 if aggregate_response.signers_operator_ids_set.contains_key(&op_info.operator_id){
//                                     non_signers_operators_ids.push(*i);
//                                 }
//                             }

//                             non_signers_operators_ids.sort_by(|a,b|{
//                                 a.cmp(b)
//                             });

//                             let mut non_signers_g1_pub_keys: Vec<G1Point> = vec![];
//                             for operator_id in non_signers_operators_ids.iter(){

//                                 if let  Some(operator) = operator_state_avs.get(operator_id){
//                                     if let Some(keys) = &operator.operator_info.pub_keys{
//                                         let g1_key = G1Point::new(u256_to_bigint256(keys.g1_pub_key.X),u256_to_bigint256(keys.g1_pub_key.Y));
//                                         non_signers_g1_pub_keys.push(g1_key);
//                                     }
//                                 }

//                             }

//                             let indices = self.avs_registry_service.get_avs_registry().get_check_signatures_indices(task_created_block,quorum_nums.clone(),non_signers_operators_ids).await.unwrap();

//                             let bls_aggregation_service_response = BlsAggregationServiceResponse{
//                                 task_index,
//                                 task_response_digest: signed_task_digest.task_response_digest,
//                                 non_signers_pub_keys_g1: non_signers_g1_pub_keys,
//                                 quorum_apks_g1: quorum_apks_g1.clone(),
//                                 signers_apk_g2: aggregate_response.signers_apk_g2,
//                                 signers_agg_sig_g1: aggregate_response.signers_agg_sig_g1,
//                                 non_signer_quorum_bitmap_indices: indices.clone().quorumApkIndices,
//                                 quorum_apk_indices: indices.quorumApkIndices,
//                                 total_stake_indices: indices.totalStakeIndices,
//                                 non_signer_stake_indices: indices.nonSignerStakeIndices

//                             };

//                             let _ = self.aggregated_response_sender.send(bls_aggregation_service_response);

//                         }

//                     }

//                 }
//             }
//         }
//     }

//     pub async fn verify_signature(
//         &self,
//         _task_index: TaskIndex,
//         signed_task_response_digest: &SignedTaskResponseDigest,
//         operator_avs_state: &HashMap<FixedBytes<32>, OperatorAvsState>,
//     ) {
//         if let Some(operator_state) =
//             operator_avs_state.get(&signed_task_response_digest.operator_id)
//         {
//             if let Some(pub_keys) = &operator_state.operator_info.pub_keys {
//                 let g2_proj = G2Point::new(
//                     (
//                         u256_to_bigint256(pub_keys.g2_pub_key.X[0]),
//                         u256_to_bigint256(pub_keys.g2_pub_key.X[1]),
//                     ),
//                     (
//                         u256_to_bigint256(pub_keys.g2_pub_key.Y[0]),
//                         u256_to_bigint256(pub_keys.g2_pub_key.Y[1]),
//                     ),
//                 )
//                 .point;
//                 let _signature_verified = signed_task_response_digest
//                     .bls_signature
//                     .verify_signature(g2_proj, &signed_task_response_digest.task_response_digest);
//             }
//         } else {
//             // throw error
//         }
//     }

//     pub fn check_if_stake_thresholds_met(
//         &self,
//         signed_stake_per_quorum: HashMap<u8, U256>,
//         total_stake_per_quorum: HashMap<u8, U256>,
//         quorum_threshold_percentages_map: HashMap<u8, QuorumThresholdPercentage>,
//     ) -> bool {
//         if let Some((quorum_num, quorum_threshold_percentage)) =
//             quorum_threshold_percentages_map.into_iter().next()
//         {
//             // to do check if quorum num <= u8 max assert
//             if let Some(signed_stake_by_quorum) = signed_stake_per_quorum.get(&quorum_num) {
//                 if let Some(total_stake_by_quorum) = total_stake_per_quorum.get(&quorum_num) {
//                     let signed_stake = signed_stake_by_quorum * U256::from(100);
//                     let threshold_stake =
//                         *total_stake_by_quorum * U256::from(quorum_threshold_percentage);
//                     return signed_stake >= threshold_stake;
//                 } else {
//                     return false;
//                 }
//             } else {
//                 return false;
//             }
//         }

//         true
//     }
// }
