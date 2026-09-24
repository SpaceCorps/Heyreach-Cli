//! Account resolution for HeyReach.
//!
//! Keys can come from a named account (`--account <name>`), a direct `--api-key <key>` flag,
//! or the `HEYREACH_API_KEY` environment variable. When named accounts are configured, `--account`
//! ensures multi-account isolation.

use crate::client::Client;
use crate::config::{self, AccountConfig, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct Resolved {
    pub name: String,
    pub config: AccountConfig,
    pub api_key: String,
}

impl Resolved {
    pub fn client(&self, verbose: bool) -> Client {
        Client::new(&self.api_key, verbose)
    }
}

pub fn resolve(requested_account: Option<&str>, direct_key: Option<&str>) -> Result<Resolved> {
    if let Some(k) = direct_key.map(str::trim).filter(|k| !k.is_empty()) {
        return Ok(Resolved {
            name: "direct".into(),
            config: AccountConfig { identity: "flag".into(), added_at: config::now_utc() },
            api_key: k.to_string(),
        });
    }

    if let Ok(k) = std::env::var("HEYREACH_API_KEY") {
        let trimmed = k.trim();
        if !trimmed.is_empty() {
            return Ok(Resolved {
                name: "env".into(),
                config: AccountConfig { identity: "HEYREACH_API_KEY".into(), added_at: config::now_utc() },
                api_key: trimmed.to_string(),
            });
        }
    }

    let config = config::load()?;

    if let Some(requested) = requested_account.map(str::trim).filter(|s| !s.is_empty()) {
        let Some((name, account)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'."))
                .detail(describe(&config))
                .fix("heyreach accounts list"));
        };

        let key = secrets::store()?.get(&secrets::account_key(name))?;
        let Some(api_key) = key.filter(|k| !k.trim().is_empty()) else {
            return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{name}' has no stored API key."))
                .detail("The config entry exists but the keystore has nothing under it.")
                .fix(format!("heyreach accounts add {name} --api-key <key>")));
        };

        return Ok(Resolved { name: name.clone(), config: account.clone(), api_key });
    }

    if config.accounts.is_empty() {
        Err(Error::new(
            ErrorCode::AuthRequired,
            "HEYREACH_API_KEY not set. Provide --api-key, set HEYREACH_API_KEY, or run 'heyreach login <name>'.",
        )
        .fix("heyreach login"))
    } else {
        Err(Error::new(
            ErrorCode::NoAccount,
            "No account specified. Pass --account <name> (or --api-key <key>, or set HEYREACH_API_KEY).",
        )
        .detail(describe(&config))
        .fix("heyreach accounts list"))
    }
}

pub fn describe(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No accounts are configured yet. Run 'heyreach accounts add <name> --api-key <key>' or 'heyreach login <name>'.".into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| if v.identity.trim().is_empty() { k.clone() } else { format!("{k} ({})", v.identity) })
        .collect();
    format!("Configured accounts: {}", listed.join(", "))
}
