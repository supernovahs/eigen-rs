use crate::{
    client::AssetID,
    contract_call::{Account, TransactionOperation},
    status::Status,
};
use serde::{Deserialize, Serialize};

/// Amount Info
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct AmountInfo {
    amount: String,
    #[serde(rename = "requestedAmount")]
    requested_amount: String,
    #[serde(rename = "netAmount")]
    net_amount: String,
    #[serde(rename = "amountUSD")]
    amount_usdc: String,
}

/// Fee Info
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct FeeInfo {
    #[serde(rename = "networkFee")]
    network_fee: String,
    #[serde(rename = "serviceFee")]
    service_fee: String,
    #[serde(rename = "gasPrice")]
    gas_price: String,
}

/// Extra Parameters
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct ExtraParameters {
    #[serde(rename = "contractCallData")]
    contract_calldata: String,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct BlockInfo {
    #[serde(rename = "blockHeight")]
    block_height: String,
    #[serde(rename = "blockHash")]
    block_hash: String,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    id: String,
    #[serde(rename = "externalId")]
    external_id: String,
    status: Status,
    #[serde(rename = "subStatus")]
    sub_status: String,
    #[serde(rename = "txHash")]
    tx_hash: String,
    operation: TransactionOperation,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "lastUpdated")]
    last_updated: i64,
    #[serde(rename = "assetId")]
    asset_id: AssetID,
    source: Account,
    #[serde(rename = "sourceAddress")]
    source_address: String,
    destination: Account,
    #[serde(rename = "DestinationAddress")]
    destination_address: String,
    #[serde(rename = "DestinationAddressDescription")]
    destination_address_description: String,
    #[serde(rename = "DestinationTag")]
    destination_tag: String,
    #[serde(rename = "amountInfo")]
    amount_info: AmountInfo,
    #[serde(rename = "feeInfo")]
    fee_info: FeeInfo,
    #[serde(rename = "feeCurrency")]
    fee_currency: String,
    #[serde(rename = "extraParameters")]
    extra_parameters: ExtraParameters,
    #[serde(rename = "numOfConfirmations")]
    num_of_confirmations: i64,
    #[serde(rename = "blockInfo")]
    block_info: BlockInfo,
}
