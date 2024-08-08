use alloy_network::Ethereum;
use alloy_primitives::Address;
use alloy_provider::Provider;
use alloy_provider::{
    fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
    RootProvider,
};
use alloy_rpc_types::Filter;
use eigen_utils::{binding::BLSApkRegistry, get_provider, NEW_PUBKEY_REGISTRATION_EVENT};
use reqwest::Client;

/// AvsRegistry Chain Subscriber struct
#[derive(Debug)]
pub struct AvsRegistryChainSubscriber {
    provider: String,
}

impl AvsRegistryChainSubscriber {
    /// New avs registry subscriber instance
    pub fn new(provider: String) -> Self {
        AvsRegistryChainSubscriber { provider }
    }

    #[allow(clippy::type_complexity)]
    /// Returns blsapkregistry instance
    pub fn build(
        &self,
        bls_apk_registry_addr: Address,
    ) -> BLSApkRegistry::BLSApkRegistryInstance<
        alloy_transport_http::Http<Client>,
        FillProvider<
            JoinFill<
                JoinFill<JoinFill<alloy_provider::Identity, GasFiller>, NonceFiller>,
                ChainIdFiller,
            >,
            RootProvider<alloy_transport_http::Http<Client>>,
            alloy_transport_http::Http<Client>,
            Ethereum,
        >,
    > {
        let provider = get_provider(&self.provider);

        BLSApkRegistry::new(bls_apk_registry_addr, provider)
    }

    /// Utility function that returns new pubkey registration filter
    pub async fn get_new_pub_key_registration_filter<'a>(
        &self,
    ) -> Result<Filter, Box<dyn std::error::Error>> {
        let provider = get_provider(&self.provider);

        let current_block_number = provider.get_block_number().await?;

        let filter = Filter::new()
            .event(NEW_PUBKEY_REGISTRATION_EVENT)
            .from_block(current_block_number);
        Ok(filter)
    }
}
