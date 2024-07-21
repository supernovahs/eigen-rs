use alloy_contract::Error as AlloyError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ElContractsError {
    /// Get slasher address
    #[error("failed to get slasher address")]
    GetSlasher,

    /// Get strategy manager
    #[error("Failed to get strategy manager address")]
    GetStrategyManager,

    /// Get delegation approval digest hash
    #[error("Failed to get delegation approval digest hash")]
    GetDelegationApprovalDigestHash,

    /// Get Operator avs registration digest hash
    #[error("Failed to get operator avs registration digest hash")]
    GetOperatorAvsRegistrationDigestHash,

    /// Get Operator shares
    #[error("Failed to get oeprator shares")]
    GetOperatorShares,

    /// Is frozen
    #[error("failed to get operator frozen status ")]
    IsFrozen,

    /// service_manager_can_slash_operator_until_block
    #[error("Failed to get service manager slashing expiry")]
    ServiceManagerCanSlashOperatorExpiry,

    /// Get underlying token
    #[error("Failed to get underlying token")]
    GetUnderlyingToken,

    /// Get operator details
    #[error("Failed to get oeprator details")]
    GetOperatorDetails,

    /// is operator or not
    #[error("Is operator or not ")]
    IsOperator,

    /// registering as operator
    #[error("Failed to register as a operator")]
    RegisterAsOperator,

    /// modify operator details
    #[error("modify operator details")]
    ModifyOperatorDetails,

    /// get strategy and underlying erc20 token
    #[error("get strategy and underlying erc20 token")]
    GetStrategyAndUnderlyingERC20Token,

    /// approve to underlying token
    #[error("Failed to call approve in underlying token contract")]
    ApproveCallToUnderlyingToken,

    /// deposit into strategy call
    #[error("Failed to deposit into strategy")]
    DepositIntoStrategy,

    /// update metadata uri
    #[error("Failed to update metadata uri")]
    UpdateMetadataUri,

    #[error("Alloy contract error: {0}")]
    AlloyContractError(#[from] AlloyError),
}
