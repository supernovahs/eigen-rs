use alloy_primitives::FixedBytes;
use eigen_crypto_bls::BlsSignature;

pub type TaskIndex = u32;

pub type TaskResponseDigest = FixedBytes<32>;

#[derive(Debug, Clone)]
pub struct SignedTaskResponseDigest {
    pub task_response_digest: TaskResponseDigest,

    pub bls_signature: BlsSignature,

    pub operator_id: FixedBytes<32>,
}
