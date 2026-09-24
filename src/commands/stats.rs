use serde_json::{Value, json};

use crate::client::Client;
use crate::error::{Error, Result};

pub fn get(
    client: &Client,
    from: Option<String>,
    to: Option<String>,
    campaign_id: Option<i64>,
    sender_id: Option<i64>,
) -> Result<Value> {
    let mut body = serde_json::Map::new();

    let account_ids: Vec<i64> = sender_id.map(|s| vec![s]).unwrap_or_default();
    let campaign_ids: Vec<i64> = campaign_id.map(|c| vec![c]).unwrap_or_default();
    body.insert("accountIds".into(), json!(account_ids));
    body.insert("campaignIds".into(), json!(campaign_ids));

    if let Some(f) = from.filter(|s| !s.trim().is_empty()) {
        validate_date(&f, "--from")?;
        body.insert("from".into(), json!(f));
    }
    if let Some(t) = to.filter(|s| !s.trim().is_empty()) {
        validate_date(&t, "--to")?;
        body.insert("to".into(), json!(t));
    }

    client.post("stats/GetOverallStats", &Value::Object(body))
}

fn validate_date(date_str: &str, field: &str) -> Result<()> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[1].chars().all(|c| c.is_ascii_digit())
        && parts[2].chars().all(|c| c.is_ascii_digit())
    {
        Ok(())
    } else {
        Err(Error::invalid(format!("{field} must be in yyyy-MM-dd format")))
    }
}
