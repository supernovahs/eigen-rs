#[derive(Debug)]
pub struct Client {}

use alloy_primitives::TxHash;

pub enum BackendType {
    HttpBAckend,
    WasmBackend,
}

#[async_trait::async_trait]
pub trait BackendClient {
    type Error;

    async fn block_number(&self, backend_type: &BackendType) -> Result<u64, Self::Error>;
    async fn block_by_number(
        &self,
        backend_type: &BackendType,
        number: u64,
    ) -> Result<Option<TxHash>, Self::Error>;
}
