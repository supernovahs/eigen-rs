#![doc(
    html_logo_url = "https://github.com/Layr-Labs/eigensdk-rs/assets/91280922/bd13caec-3c00-4afc-839a-b83d2890beb5",
    issue_tracker_base_url = "https://github.com/Layr-Labs/eigensdk-rs/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod binding;
use alloy_json_rpc::RpcError;
use alloy_network::{Ethereum, EthereumWallet};
use alloy_provider::{
    fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
    ProviderBuilder, RootProvider, WsConnect,
};
use alloy_pubsub::PubSubFrontend;
use alloy_signer_local::PrivateKeySigner;
use alloy_transport::TransportErrorKind;
use alloy_transport_http::{Client, Http};
use reqwest::Url;
use std::fs;
pub fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap()
}

#[allow(clippy::type_complexity)]
pub fn get_signer(
    key: String,
    rpc_url: &str,
) -> FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<JoinFill<alloy_provider::Identity, GasFiller>, NonceFiller>,
            ChainIdFiller,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
> {
    let signer: PrivateKeySigner = key.parse().expect("failed to generate wallet ");
    let wallet = EthereumWallet::from(signer);
    let url = Url::parse(rpc_url).expect("Wrong rpc url");
    ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet.clone())
        .on_http(url)
}

#[allow(clippy::type_complexity)]
pub fn get_provider(
    rpc_url: &str,
) -> FillProvider<
    JoinFill<JoinFill<JoinFill<alloy_provider::Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
> {
    let url = Url::parse(rpc_url).expect("Wrong rpc url");
    ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(url)
}

#[allow(clippy::type_complexity)]
pub async fn get_ws_provider(
    rpc_url: &str,
) -> Result<RootProvider<PubSubFrontend>, RpcError<TransportErrorKind>> {
    let ws = WsConnect::new(rpc_url);
    ProviderBuilder::new().on_ws(ws).await
}

/// Emitted when a new pubkey is registered
pub const NEW_PUBKEY_REGISTRATION_EVENT: &str =
    "NewPubkeyRegistration(address,(uint256,uint256),(uint256[2],uint256[2]))";
