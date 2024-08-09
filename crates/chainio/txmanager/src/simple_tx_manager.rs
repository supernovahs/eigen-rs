use alloy_eips::BlockNumberOrTag;
use alloy_network::{Ethereum, EthereumWallet, TransactionBuilder};
use alloy_primitives::Address;
use alloy_provider::{PendingTransactionBuilder, Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types_eth::{TransactionInput, TransactionReceipt, TransactionRequest};
use alloy_signer_local::PrivateKeySigner;
use eigen_logging::{logger::Logger, tracing_logger::TracingLogger};
use eigen_signer::signer::Config;
use k256::ecdsa::SigningKey;
use reqwest::Url;
use thiserror::Error;

static FALLBACK_GAS_TIP_CAP: u128 = 5_000_000_000;

pub type Transport = alloy_transport_http::Http<reqwest::Client>;

/// Possible errors raised in Tx Manager
#[derive(Error, Debug)]
pub enum TxManagerError {
    #[error("signer error")]
    SignerError,
    #[error("send error")]
    SendTxError,
    #[error("wait_for_receipt error")]
    WaitForReceiptError,
    #[error("address error")]
    AddressError,
    #[error("invalid url error")]
    InvalidUrlError,
}

pub struct SimpleTxManager<'log> {
    logger: &'log TracingLogger,
    gas_limit_multiplier: f64,
    private_key: String,
    provider: RootProvider<Transport>,
}

impl<'log> SimpleTxManager<'log> {
    /// Creates a new SimpleTxManager.
    ///
    /// # Arguments
    ///
    /// - `logger`: The logger to be used.
    /// - `gas_limit_multiplier`: The gas limit multiplier.
    /// - `private_key`: The private key of the wallet.
    /// - `rpc_url`: The RPC URL. It could be an anvil node or any other node.
    ///
    /// # Returns
    ///
    /// - The SimpleTxManager created.
    ///
    /// # Errors
    ///
    /// - If the URL is invalid.
    pub fn new(
        logger: &'log TracingLogger,
        gas_limit_multiplier: f64,
        private_key: &str,
        rpc_url: &str,
    ) -> Result<SimpleTxManager<'log>, TxManagerError> {
        let url = Url::parse(rpc_url)
            .inspect_err(|err| logger.error("Failed to parse url", &[err]))
            .map_err(|_| TxManagerError::InvalidUrlError)?;
        let provider = ProviderBuilder::new().on_http(url);
        Ok(SimpleTxManager {
            logger,
            gas_limit_multiplier,
            private_key: private_key.to_string(),
            provider,
        })
    }

    /// Returns the address of the wallet, beloing to the given private key.
    ///
    /// # Returns
    ///
    /// - The address of the wallet.
    ///
    /// # Errors
    ///
    /// - If the private key is invalid.
    pub fn get_address(&self) -> Result<Address, TxManagerError> {
        let private_key_signing_key = SigningKey::from_slice(self.private_key.as_bytes())
            .inspect_err(|err| self.logger.error("Failed to parse private key", &[err]))
            .map_err(|_| TxManagerError::AddressError)?;
        Ok(Address::from_private_key(&private_key_signing_key))
    }

    pub fn with_gas_limit_multiplier(&mut self, multiplier: f64) {
        self.gas_limit_multiplier = multiplier;
    }

    fn create_local_signer(&self) -> Result<PrivateKeySigner, TxManagerError> {
        let config = Config::PrivateKey(self.private_key.clone());
        Config::signer_from_config(config)
            .inspect_err(|err| self.logger.error("Failed to create signer", &[err]))
            .map_err(|_| TxManagerError::SignerError)
    }

    /// Send is used to send a transaction to the Ethereum node. It takes an unsigned/signed transaction
    /// and then sends it to the Ethereum node.
    /// If you pass in a signed transaction it will ignore the signature
    /// and resign the transaction after adding the nonce and gas limit.
    ///
    /// # Arguments
    ///
    /// - `tx`: The transaction to be sent.
    ///
    /// # Returns
    ///
    /// - The transaction receipt.
    pub async fn send_tx(
        &self,
        tx: &mut TransactionRequest,
    ) -> Result<TransactionReceipt, TxManagerError> {
        // Estimating gas and nonce
        self.logger.debug("Estimating gas and nonce", &[()]);

        let tx = self
            .estimate_gas_and_nonce(tx)
            .await
            .inspect_err(|err| self.logger.error("Failed to estimate gas", &[err]))?;

        let signer = self.create_local_signer()?;
        let wallet = EthereumWallet::from(signer);

        let signed_tx = tx
            .build(&wallet)
            .await
            .inspect_err(|err| {
                self.logger
                    .error("Failed to build and sign transaction", &[err])
            })
            .map_err(|_| TxManagerError::SendTxError)?;

        // send transaction and get receipt
        let pending_tx = self
            .provider
            .send_transaction(signed_tx.into())
            .await
            .inspect_err(|err| self.logger.error("Failed to get receipt", &[err]))
            .map_err(|_| TxManagerError::SendTxError)?;

        self.logger
            .debug("Transaction sent. Pending transaction: ", &[&pending_tx]);

        // wait for the transaction to be mined
        SimpleTxManager::wait_for_receipt(self, pending_tx).await
    }

    /// Estimates the gas and nonce for a transaction.
    ///
    /// # Arguments
    ///
    /// - `tx`: The transaction for which we want to estimate the gas and nonce.
    ///
    /// # Returns
    ///
    /// - The transaction request with the gas and nonce estimated.
    ///
    /// # Errors
    ///
    /// - If the transaction request could not sent of gives an error.
    /// - If the latest block header could not be retrieved.
    /// - If the gas price could not be estimated.
    /// - If the gas limit could not be estimated.
    /// - If the destination address could not be retrieved.
    async fn estimate_gas_and_nonce(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionRequest, TxManagerError> {
        let gas_tip_cap = self.provider.get_max_priority_fee_per_gas().await
        .inspect_err(|err|
            self.logger.info("eth_maxPriorityFeePerGas is unsupported by current backend, using fallback gasTipCap",
            &[err]))
        .unwrap_or(FALLBACK_GAS_TIP_CAP);

        let header = self
            .provider
            .get_block_by_number(BlockNumberOrTag::Latest, false)
            .await
            .ok()
            .flatten()
            .map(|block| block.header)
            .ok_or(TxManagerError::SendTxError)
            .inspect_err(|_| {
                self.logger
                    .error("Failed to get latest block header", &[()])
            })?;

        // 2*baseFee + gas_tip_cap makes sure that the tx remains includeable for 6 consecutive 100% full blocks.
        // see https://www.blocknative.com/blog/eip-1559-fees
        let base_fee = header.base_fee_per_gas.ok_or(TxManagerError::SendTxError)?;
        let gas_fee_cap = 2 * base_fee + gas_tip_cap;

        let mut gas_limit = tx.gas_limit();
        let tx_input = tx.input().unwrap_or_default().to_vec();
        // we only estimate if gas_limit is not already set
        if let Some(0) = gas_limit {
            let from = self.get_address()?;
            let to = match tx.to() {
                Some(c) => c,
                None => return Err(TxManagerError::SendTxError),
            };
            let mut tx_request = TransactionRequest::default()
                .to(to)
                .from(from)
                .value(tx.value().unwrap_or_default())
                .input(TransactionInput::new(tx_input.clone().into()));
            tx_request.set_max_priority_fee_per_gas(gas_tip_cap);
            tx_request.set_max_fee_per_gas(gas_fee_cap);

            gas_limit = Some(
                self.provider
                    .estimate_gas(&tx_request)
                    .await
                    .map_err(|_| TxManagerError::SendTxError)?,
            );
        }
        let gas_price_multiplied =
            tx.gas_price().unwrap_or_default() as f64 * self.gas_limit_multiplier;
        let gas_price = gas_price_multiplied as u128;

        let to = match tx.to() {
            None => return Err(TxManagerError::SendTxError),
            Some(adress) => adress,
        };

        let new_tx = TransactionRequest::default()
            .with_to(to)
            .with_value(tx.value().unwrap_or_default())
            .with_gas_limit(gas_limit.unwrap_or_default())
            .with_nonce(tx.nonce().unwrap_or_default())
            .with_input(tx_input)
            .with_chain_id(tx.chain_id().unwrap_or(1))
            .with_max_priority_fee_per_gas(gas_tip_cap)
            .with_max_fee_per_gas(gas_fee_cap)
            .with_gas_price(gas_price);

        Ok(new_tx)
    }

    /// Waits for the transaction receipt.
    ///
    /// This is a wrapper around `PendingTransactionBuilder::get_receipt`.
    ///
    /// # Arguments
    ///
    /// - `pending_tx`: The pending transaction builder we want to wait for.
    ///
    /// # Returns
    ///
    /// - The block number in which the transaction was included.
    /// - `None` if the transaction was not included in a block or an error ocurred.
    pub async fn wait_for_receipt(
        &self,
        pending_tx: PendingTransactionBuilder<'_, Transport, Ethereum>,
    ) -> Result<TransactionReceipt, TxManagerError> {
        pending_tx
            .get_receipt()
            .await
            .inspect_err(|err| self.logger.error("Failed to get receipt", &[err]))
            .map_err(|_| TxManagerError::WaitForReceiptError)
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleTxManager;
    use alloy_consensus::TxLegacy;
    use alloy_network::TransactionBuilder;
    use alloy_node_bindings::Anvil;
    use alloy_primitives::{bytes, TxKind::Call, U256};
    use alloy_rpc_types_eth::TransactionRequest;
    use eigen_logging::{log_level::LogLevel, logger::Logger, tracing_logger::TracingLogger};
    use once_cell::sync::OnceCell;
    use tokio;

    static TEST_LOGGER: OnceCell<TracingLogger> = OnceCell::new();
    const PRIVATE_KEY: &str = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7";

    #[tokio::test]
    async fn test_send_transaction_from_legacy() {
        TEST_LOGGER.get_or_init(|| {
            TracingLogger::new_text_logger(false, String::from(""), LogLevel::Debug, false)
        });

        // Spin up a local Anvil node.
        // Ensure `anvil` is available in $PATH.
        let anvil = Anvil::new().try_spawn().unwrap();
        let rpc_url: String = anvil.endpoint().parse().unwrap();

        // Create a provider.
        let logger = TEST_LOGGER.get().unwrap();
        let simple_tx_manager =
            SimpleTxManager::new(logger, 1.0, PRIVATE_KEY, rpc_url.as_str()).unwrap();

        // Create two users, Alice and Bob.
        let _alice = anvil.addresses()[0];
        let bob = anvil.addresses()[1];

        // Test 1: legacy tx
        let tx = TxLegacy {
            to: Call(bob),
            value: U256::from(1_000_000_000),
            gas_limit: 2_000_000,
            nonce: 0,
            gas_price: 21_000_000_000,
            input: bytes!(),
            chain_id: Some(31337),
        };

        let mut tx_request: TransactionRequest = tx.clone().into();
        //// send transaction and wait for receipt
        let receipt = simple_tx_manager.send_tx(&mut tx_request).await.unwrap();
        let block_number = receipt.block_number.unwrap();
        println!("Transaction mined in block: {}", block_number);
        assert!(block_number > 0);
        assert_eq!(receipt.to, Some(bob));
    }

    #[tokio::test]
    async fn test_send_transaction_from_eip1559() {
        TEST_LOGGER.get_or_init(|| {
            TracingLogger::new_text_logger(false, String::from(""), LogLevel::Debug, false)
        });

        // Spin up a local Anvil node.
        // Ensure `anvil` is available in $PATH.
        let anvil = Anvil::new().try_spawn().unwrap();
        let rpc_url: String = anvil.endpoint().parse().unwrap();

        // Create a provider.
        let logger = TEST_LOGGER.get().unwrap();
        let simple_tx_manager =
            SimpleTxManager::new(logger, 1.0, PRIVATE_KEY, rpc_url.as_str()).unwrap();

        // Create two users, Alice and Bob.
        let _alice = anvil.addresses()[0];
        let bob = anvil.addresses()[1];

        let mut tx = TransactionRequest::default()
            .with_to(bob)
            .with_nonce(0)
            .with_chain_id(anvil.chain_id())
            .with_value(U256::from(100))
            .with_gas_limit(21_000)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);

        tx.set_gas_price(21_000_000_000);
        // send transaction and wait for receipt
        let receipt = simple_tx_manager.send_tx(&mut tx).await.unwrap();
        let block_number = receipt.block_number.unwrap();
        println!("Transaction mined in block: {}", block_number);
        assert!(block_number > 0);
        assert_eq!(receipt.to, Some(bob));
    }
}
