use alloy_primitives::FixedBytes;
use eigen_crypto_bls::Signature;

pub type TaskIndex = u32;

pub type TaskResponseDigest = FixedBytes<32>;

#[derive(Debug, Clone)]
pub struct SignedTaskResponseDigest {
    pub task_response_digest: TaskResponseDigest,

    pub bls_signature: Signature,

    pub operator_id: FixedBytes<32>,
}
