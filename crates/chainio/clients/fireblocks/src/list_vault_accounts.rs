use crate::client::{AssetID, Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Asset {
    id: AssetID,
    total: String,
    balance: String,
    available: String,
    locked_amount: String,
    pending: String,
    frozen: String,
    staked: String,
    block_height: String,
    #[serde(default)]
    block_hash: Option<String>,
}

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultAccount {
    id: String,
    name: String,
    #[serde(rename = "hiddenOnUI")]
    hidden_on_ui: bool,
    #[serde(rename = "autoFuel")]
    auto_fuel: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultAccountResponse {
    #[serde(rename = "accounts")]
    vault_accounts: Vec<VaultAccount>,
    paging: Paging,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Paging {}

/// Get List Vault Accounts trait for "/v1/vault/accounts_paged" get requests
#[allow(unused)]
pub trait ListVaultAccounts {
    async fn list_vault_accounts(&self) -> Result<VaultAccountResponse, String>;
}

impl ListVaultAccounts for Client {
    async fn list_vault_accounts(&self) -> Result<VaultAccountResponse, String> {
        let list_vault_accounts_result =
            self.get_request(&format!("/v1/vault/accounts_paged")).await;

        match list_vault_accounts_result {
            Ok(list_vault_accounts) => {
                println!("list vault accounts {:?}", list_vault_accounts);
                let vault_accounts: VaultAccountResponse =
                    serde_json::from_str(&list_vault_accounts).unwrap();
                return Ok(vault_accounts);
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    #[cfg(feature = "fireblock-tests")]
    async fn test_list_vault_accounts() {
        let api_key = env::var("FIREBLOCKS_API_KEY").expect("FIREBLOCKS_API_KEY not set");
        let private_key_path =
            env::var("FIREBLOCKS_PRIVATE_KEY_PATH").expect("FIREBLOCKS_PRIVATE_KEY_PATH not set");
        let api_url = env::var("FIREBLOCKS_API_URL").expect("FIREBLOCKS_API_URL not set");
        let private_key =
            std::fs::read_to_string(private_key_path).expect("Failed to read private key file");
        let client = Client::new(
            api_key.to_string(),
            private_key.to_string(),
            api_url.clone(),
        );

        let _ = client.list_vault_accounts().await.unwrap();
    }
}
