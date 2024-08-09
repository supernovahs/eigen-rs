use alloy_sol_types::sol;

// https://github.com/Layr-Labs/eigenlayer-middleware/blob/m2-mainnet/src/StakeRegistry.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    StakeRegistry,
    "../../crates/contracts/bindings/utils/json/StakeRegistry.json"
);

// https://github.com/Layr-Labs/eigenlayer-middleware/blob/m2-mainnet/src/RegistryCoordinator.sol
sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    RegistryCoordinator,
    "../../crates/contracts/bindings/utils/json/RegistryCoordinator.json"
);

// https://github.com/Layr-Labs/eigenlayer-middleware/blob/m2-mainnet/src/OperatorStateRetriever.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    OperatorStateRetriever,
    "../../crates/contracts/bindings/utils/json/OperatorStateRetriever.json"
);

// https://github.com/Layr-Labs/eigenlayer-middleware/blob/m2-mainnet/src/BLSApkRegistry.sol
sol!(
    #[allow(missing_docs)]
    #[derive(Debug)]
    #[sol(rpc)]
    BLSApkRegistry,
    "../../crates/contracts/bindings/utils/json/BLSApkRegistry.json"
);

// https://github.com/Layr-Labs/eigenlayer-middleware/blob/m2-mainnet/src/ServiceManagerBase.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ServiceManagerBase,
    "../../crates/contracts/bindings/utils/json/ServiceManagerBase.json"
);

// https://github.com/Layr-Labs/eigenlayer-contracts/blob/mainnet/src/contracts/core/AVSDirectory.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AVSDirectory,
    "../../crates/contracts/bindings/utils/json/AVSDirectory.json"
);

// https://github.com/Layr-Labs/eigenlayer-contracts/blob/mainnet/src/contracts/core/DelegationManager.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    DelegationManager,
    "../../crates/contracts/bindings/utils/json/DelegationManager.json"
);

// https://github.com/Layr-Labs/eigenlayer-contracts/blob/mainnet/src/contracts/interfaces/ISlasher.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ISlasher,
    "../../crates/contracts/bindings/utils/json/ISlasher.json"
);

// https://github.com/Layr-Labs/eigenlayer-contracts/blob/mainnet/src/contracts/interfaces/IStrategy.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IStrategy,
    "../../crates/contracts/bindings/utils/json/IStrategy.json"
);

// standard
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IERC20,
    "../../crates/contracts/bindings/utils/json/IERC20.json"
);

// https://github.com/Layr-Labs/eigenlayer-contracts/blob/mainnet/src/contracts/core/StrategyManager.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    StrategyManager,
    "../../crates/contracts/bindings/utils/json/StrategyManager.json"
);

// https://github.com/Layr-Labs/eigenlayer-contracts/blob/mainnet/src/contracts/interfaces/IAVSDirectory.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IAVSDirectory,
    "../../crates/contracts/bindings/utils/json/IAVSDirectory.json"
);

// https://github.com/Layr-Labs/eigenlayer-middleware/blob/mainnet/src/unaudited/ECDSAStakeRegistry.sol
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ECDSAStakeRegistry,
    "../../crates/contracts/bindings/utils/json/ECDSAStakeRegistry.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IBLSSignatureChecker,
    "../../crates/contracts/bindings/utils/json/IBLSSignatureChecker.json"
);

// Anvil utilities

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ContractsRegistry,
    "../../crates/contracts/bindings/utils/json/ContractsRegistry.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    mockAvsServiceManager,
    "../../crates/contracts/bindings/utils/json/MockAvsServiceManager.json"
);
