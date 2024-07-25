use serde::{Deserialize, Serialize};

use crate::{
    client::AssetID,
    contract_call::{Account, ExtraParams, TransactionOperation},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    operation: String,

    external_tx_id: String,

    asset_id: AssetID,

    source: Account,

    destination: Account,

    amount: String,

    extra_parameters: ExtraParams,

    replace_tx_by_hash: String,

    gas_price: String,

    gas_limit: String,

    max_fee: String,

    priority_fee: String,

    fee_level: String,
}

impl TransactionRequest {
    pub fn new(
        operation: String,

        external_tx_id: String,

        asset_id: AssetID,

        source: Account,

        destination: Account,

        amount: String,

        extra_parameters: ExtraParams,

        replace_tx_by_hash: String,

        gas_price: String,

        gas_limit: String,

        max_fee: String,

        priority_fee: String,

        fee_level: String,
    ) -> Self {
        Self {
            operation,
            external_tx_id,
            asset_id,
            source,
            destination,
            amount,
            extra_parameters,
            replace_tx_by_hash,
            gas_price,
            gas_limit,
            max_fee,
            priority_fee,
            fee_level,
        }
    }

    pub fn get_contract_call(&mut self) -> Self {
        self.operation = TransactionOperation::contract_call.as_str().to_string();
        Self {
            operation: self.operation.clone(),
            external_tx_id: self.external_tx_id.clone(),
            asset_id: self.asset_id.clone(),
            source: self.source.clone(),
            destination: self.destination.clone(),
            amount: self.amount.clone(),
            extra_parameters: self.extra_parameters.clone(),
            replace_tx_by_hash: self.replace_tx_by_hash.clone(),
            gas_price: self.gas_price.clone(),
            gas_limit: self.gas_limit.clone(),
            max_fee: self.max_fee.clone(),
            priority_fee: self.priority_fee.clone(),
            fee_level: self.fee_level.clone(),
        }
    }
}

impl std::fmt::Display for TransactionRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TransactionRequest {{
    operation: {},
    external_tx_id: {},
    asset_id: {},
    source: {},
    destination: {},
    amount: {},
    extra_parameters: {},
    replace_tx_by_hash: {},
    gas_price: {},
    gas_limit: {},
    max_fee: {},
    priority_fee: {},
    fee_level: {}
}}",
            self.operation,
            self.external_tx_id,
            self.asset_id,
            self.source,
            self.destination,
            self.amount,
            self.extra_parameters,
            self.replace_tx_by_hash,
            self.gas_price,
            self.gas_limit,
            self.max_fee,
            self.priority_fee,
            self.fee_level
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    id: String,
    status: String,
}