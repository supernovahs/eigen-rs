use alloy_primitives::Address;
use alloy_provider::{Provider, ProviderBuilder, WsConnect};
use alloy_rpc_types::Filter;
use anyhow::Result;
use eigen_client_avsregistry::reader::AvsRegistryChainReader;
use eigen_logging::{logger::Logger, tracing_logger::TracingLogger};
use eigen_types::operator::{operator_id_from_g1_pub_key, OperatorPubKeys};
use eigen_utils::{
    binding::BLSApkRegistry::{self, G1Point, G2Point},
    NEW_PUBKEY_REGISTRATION_EVENT,
};
use futures_util::StreamExt;
use std::collections::HashMap;
use tokio::sync::{
    mpsc,
    mpsc::UnboundedSender,
    oneshot::{self, Sender},
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OperatorInfoServiceInMemory {
    logger: TracingLogger,
    avs_registry_reader: AvsRegistryChainReader,
    ws: String,
    pub_keys: UnboundedSender<OperatorsInfoMessage>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum OperatorsInfoMessage {
    InsertOperatorInfo(Address, OperatorPubKeys),
    Remove(Address),
    Get(Address, Sender<Option<OperatorPubKeys>>),
}

impl OperatorInfoServiceInMemory {
    pub async fn new(
        logger: TracingLogger,
        avs_registry_chain_reader: AvsRegistryChainReader,
        web_socket: String,
    ) -> Self {
        let (pubkeys_tx, mut pubkeys_rx) = mpsc::unbounded_channel();
        let mut operator_info_data = HashMap::new();

        let mut operator_addr_to_id = HashMap::new();

        tokio::spawn(async move {
            while let Some(cmd) = pubkeys_rx.recv().await {
                match cmd {
                    OperatorsInfoMessage::InsertOperatorInfo(addr, keys) => {
                        operator_info_data.insert(addr, keys.clone());
                        let operator_id = operator_id_from_g1_pub_key(keys.g1_pub_key);
                        operator_addr_to_id.insert(addr, operator_id);
                    }
                    OperatorsInfoMessage::Remove(addr) => {
                        operator_info_data.remove(&addr);
                    }
                    OperatorsInfoMessage::Get(addr, responder) => {
                        let result = operator_info_data.get(&addr).cloned();
                        let _ = responder.send(result);
                    }
                }
            }
        });

        Self {
            logger,
            avs_registry_reader: avs_registry_chain_reader,
            ws: web_socket,
            pub_keys: pubkeys_tx,
        }
    }

    pub async fn start_service(
        &self,
        start_block: u64,
        end_block: u64,
        shutdown_rx: tokio::sync::watch::Receiver<()>,
    ) -> Result<()> {
        // query past operator registrations
        self.query_past_registered_operator_events_and_fill_db(
            start_block,
            end_block,
            self.ws.clone(),
        )
        .await;

        let ws = WsConnect::new(&self.ws);
        let provider = ProviderBuilder::new().on_ws(ws).await?;
        let current_block_number = provider.get_block_number().await?;
        let filter = Filter::new()
            .event(NEW_PUBKEY_REGISTRATION_EVENT)
            .from_block(current_block_number);

        let subcription_new_operator_registration_stream = provider.subscribe_logs(&filter).await?;
        let mut stream = subcription_new_operator_registration_stream.into_stream();

        let shutdown_rx = shutdown_rx.clone();
        let pub_keys = self.pub_keys.clone();
        let self_clone = self.clone();
        tokio::spawn(async move {
            while let Some(log) = stream.next().await {
                if shutdown_rx.has_changed().unwrap_or(false) {
                    self_clone.logger.info(
                        "shutdown of operators info service ",
                        &["eigen-services-operatorsinfo.start_service"],
                    );
                    break;
                }

                let data = log
                    .log_decode::<BLSApkRegistry::NewPubkeyRegistration>()
                    .ok();

                if let Some(new_pub_key_event) = data {
                    let event_data = new_pub_key_event.data();
                    let operator_pub_key = OperatorPubKeys {
                        g1_pub_key: G1Point {
                            X: event_data.pubkeyG1.X,
                            Y: event_data.pubkeyG1.Y,
                        },
                        g2_pub_key: G2Point {
                            X: event_data.pubkeyG2.X,
                            Y: event_data.pubkeyG2.Y,
                        },
                    };
                    // Send message
                    self_clone.logger.debug(
                        &format!(
                            "New pub key found  operator_address : {:?} , operator_pub_keys : {:?}",
                            event_data.operator, operator_pub_key
                        ),
                        &["eigen-services-operatorsinfo.start_service"],
                    );
                    let _ = pub_keys.send(OperatorsInfoMessage::InsertOperatorInfo(
                        event_data.operator,
                        operator_pub_key,
                    ));
                }
            }
        });

        Ok(())
    }

    pub async fn get_operator_info(&self, address: Address) -> Option<OperatorPubKeys> {
        let (responder_tx, responder_rx) = oneshot::channel();
        let _ = self
            .pub_keys
            .send(OperatorsInfoMessage::Get(address, responder_tx));
        responder_rx.await.unwrap_or(None)
    }

    pub async fn query_past_registered_operator_events_and_fill_db(
        &self,
        start_block: u64,
        end_block: u64,
        ws_url: String,
    ) {
        let (operator_address, operator_pub_keys) = self
            .avs_registry_reader
            .query_existing_registered_operator_pub_keys(start_block, end_block, ws_url)
            .await
            .unwrap();
        for (i, address) in operator_address.iter().enumerate() {
            let message =
                OperatorsInfoMessage::InsertOperatorInfo(*address, operator_pub_keys[i].clone());
            self.logger.debug(&format!("New pub key found  operator_address : {:?} , operator_pub_keys : {:?}",operator_address,operator_pub_keys), &["eigen-services-operatorsinfo.query_past_registered_operator_events_and_fill_db"]);
            let _ = self.pub_keys.send(message);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloy_primitives::address;
    use eigen_logging::get_test_logger;
    use eigen_testing_utils::m2_holesky_constants::{
        OPERATOR_STATE_RETRIEVER, REGISTRY_COORDINATOR,
    };
    use std::env;
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_query_past_registered_operator_events_and_fill_db() {
        let websocket_url_holesky = env::var("HOLESKY_WS_URL").expect("HOLEESKY_WS_URL not set");
        let http_url_holesky = env::var("HOLESKY_HTTP_URL").expect("HOLESKY_HTTP_URL not set");
        let test_logger = get_test_logger();
        let avs_registry_chain_reader = AvsRegistryChainReader::new(
            test_logger.clone(),
            REGISTRY_COORDINATOR,
            OPERATOR_STATE_RETRIEVER,
            http_url_holesky.clone(),
        )
        .await
        .unwrap();
        let operators_info_service_in_memory = OperatorInfoServiceInMemory::new(
            test_logger.clone(),
            avs_registry_chain_reader,
            websocket_url_holesky.clone(),
        )
        .await;

        operators_info_service_in_memory
            .query_past_registered_operator_events_and_fill_db(
                2019065,
                2039045,
                websocket_url_holesky,
            )
            .await;

        // Give some time for the background task to process the messages
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let address = address!("385b3b04126b221b09ad68fd55dee74965e9be8b");
        let operator_info = operators_info_service_in_memory
            .get_operator_info(address)
            .await;
        assert!(operator_info.is_some());
    }

    #[tokio::test]
    async fn test_start_service() {
        let websocket_url_holesky = env::var("HOLESKY_WS_URL").expect("HOLEESKY_WS_URL not set");
        let http_url_holesky = env::var("HOLESKY_HTTP_URL").expect("HOLESKY_HTTP_URL not set");
        let test_logger = get_test_logger();
        let avs_registry_chain_reader = AvsRegistryChainReader::new(
            test_logger.clone(),
            REGISTRY_COORDINATOR,
            OPERATOR_STATE_RETRIEVER,
            http_url_holesky.clone(),
        )
        .await
        .unwrap();
        let operators_info_service_in_memory = OperatorInfoServiceInMemory::new(
            test_logger.clone(),
            avs_registry_chain_reader,
            websocket_url_holesky.clone(),
        )
        .await;

        let (shutdown_tx, shutdown_rx) = watch::channel(());

        // Use a timeout to ensure the test does not run indefinitely
        let _ = tokio::spawn({
            async move {
                let shutdown_rx = shutdown_rx.clone();
                operators_info_service_in_memory
                    .start_service(2019065, 2039045, shutdown_rx)
                    .await
                    .unwrap();
            }
        });

        // Wait some time to simulate some operations
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // // Send the shutdown signal
        let _ = shutdown_tx.send(());

        // TODO (supernova): Call Register Operator to emit the NewPubkeyRegistration event , so the subscriber catches it.
        // Blocked : https://github.com/Layr-Labs/eigensdk-rs/issues/49
        // // Wait for the service to finish
        // let _ = service_handle.await;
    }
}
