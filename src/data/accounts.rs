use crate::error::Result;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Account {
    pub profile_id: String,
    pub username: String,
    pub is_active: bool,
}

#[derive(Deserialize)]
struct AccountsFile {
    accounts: Vec<AccountEntry>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AccountEntry {
    #[serde(rename = "entitlementOwned")]
    entitlement_owned: Option<bool>,
    #[serde(rename = "localId")]
    local_id: Option<String>,
    // Note: The field is "profile" in the JSON, not "minecraftProfile"
    profile: Option<MinecraftProfile>,
    active: Option<bool>,
}

#[derive(Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

pub fn load_accounts(accounts_path: &PathBuf) -> Result<Vec<Account>> {
    if !accounts_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(accounts_path)?;
    let accounts_file: AccountsFile = serde_json::from_str(&content)?;

    let accounts = accounts_file
        .accounts
        .into_iter()
        .filter_map(|entry| {
            let profile = entry.profile?;
            Some(Account {
                profile_id: profile.id,
                username: profile.name,
                is_active: entry.active.unwrap_or(false),
            })
        })
        .collect();

    Ok(accounts)
}
