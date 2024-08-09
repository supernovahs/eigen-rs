use alloy_contract::Error as AlloyError;
use eigen_client_elcontracts::error::ElContractsError;
use eigen_crypto_bls::error::BlsError;
use thiserror::Error;

/// Error returned by AvsRegistry
#[derive(Debug, Error)]
pub enum AvsRegistryError {
    /// Could not get bls apk registry addresss
    #[error("Failed to get bls apk registry address")]
    GetBlsApkRegistry,

    /// Failed to get quorum count
    #[error("Faield to get quorum count")]
    GetQuorumCount,

    /// Failed to ger operator state
    #[error("Failed to get operator state")]
    GetOperatorState,

    /// Failed to get operator state with registry coordinator and operator id
    #[error("Failed to get oeprator state with registry coordinator and operator id ")]
    GetOperatorStateWithRegistryCoordinatorAndOperatorId,

    /// Failed to get current block number
    #[error("Failed to get current block number")]
    GetBlockNumber,

    /// block number overflow
    #[error("block number overflowed")]
    BlockNumberOverflow,

    /// Operator stake in quourm at block
    #[error("failed to get operator stake in quorum at block number")]
    GetOperatorStakeInQuorumAtBlockNumber,

    /// Operator stake in quorums at block operator id
    #[error("Failed to get operator stake in quorums at block operator id ")]
    GetOperatorStakeInQuorumAtBlockOperatorId,

    /// Operator stake in quourm at block
    #[error("failed to get operator stake in quorum at current block number")]
    GetOperatorStakeInQuorumAtCurrentBlockNumber,

    /// Current Quorum bitmap
    #[error("Failed to get current quorum bitmap")]
    GetCurrentQuorumBitmap,

    /// Current stake
    #[error("Failed to get current stake")]
    GetCurrentStake,

    /// Check Signature indices result
    #[error("Check Signature indices result")]
    CheckSignatureIndices,

    /// Get Operator Id
    #[error("Get Operator Id")]
    GetOperatorId,

    /// Get Operator from Id
    #[error("Get Operator from Id")]
    GetOperatorFromId,

    /// Get Operator Status
    #[error("Get Operator Status")]
    GetOperatorStatus,

    /// Eth logs query
    #[error("Failed to get eth_logs")]
    GetEthLogs,

    /// Failed to decode event
    #[error("Failed to decode New PubKey Registration Filter")]
    DecodeEventNewPubkeyRegistrationFilter,

    /// Failed to decode event
    #[error("failed to decode  operator socket update filter ")]
    DecodeEventOperatorSocketUpdateFilter,

    /// Failed to get Service Manager
    #[error("Failed to get service manager")]
    GetServiceManager,

    /// Failed to get stake registry
    #[error("Failed to get stake registry address")]
    GetStakeRegistry,

    /// Failed to get delegation manager address
    #[error("Failed to get delegation manager address")]
    GetDelegation,

    /// Failed to get avs registry address
    #[error("failed to get avs registry ")]
    GetAvsRegistry,

    /// Pubey registration msg hash
    #[error("Failed to regiser pub key message hash")]
    PubKeyRegistrationMessageHash,

    /// Failed to calculate operator avs registration digest hash
    #[error(" Failed to calculate operator avs registration digest hash")]
    CalculateOperatorAvsRegistrationDigestHash,

    /// Register Operator
    #[error("Failed to register operatror")]
    RegisterOperator,

    /// Update Operator for Quorum
    #[error("Failed to update operator for quorum")]
    UpdateOperatorForQuorum,

    /// Failed to Deregister operator
    #[error("failed to deregister operator")]
    DeregisterOperator,

    /// build el chain reader
    #[error("Failed to build el chain reader")]
    BuildElChainReader,

    /// Update stake for all quorums
    #[error("Failed to update stake foor all quorums")]
    UpdateStakeForAllQuorums,

    /// Failed to get g2 projective
    #[error("pub key g2")]
    PUbKeyG2,

    /// Failed to subscribe to logs
    #[error("Could not subscribe to logs ")]
    SubscribeLogs,

    /// Alloy errors
    #[error("Alloy contract error: {0}")]
    AlloyContractError(#[from] AlloyError),

    /// ElContractsError compatibility
    #[error("ElContractsError: {0}")]
    ElContractsError(String),

    /// BlsError compatibility
    #[error("BlsError :{0}")]
    BlsError(String),
}

impl From<ElContractsError> for AvsRegistryError {
    fn from(err: ElContractsError) -> Self {
        AvsRegistryError::ElContractsError(err.to_string())
    }
}

impl From<BlsError> for AvsRegistryError {
    fn from(err: BlsError) -> Self {
        AvsRegistryError::BlsError(err.to_string())
    }
}
