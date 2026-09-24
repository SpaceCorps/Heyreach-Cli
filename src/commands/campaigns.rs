use std::io::{IsTerminal, Read};

use serde_json::{Value, json};

use crate::client::Client;
use crate::error::{Error, Result};
use crate::obj;

pub fn list(client: &Client, limit: u32, offset: u32, status: Option<String>) -> Result<Value> {
    let mut body = serde_json::Map::new();
    body.insert("offset".into(), json!(offset));
    body.insert("limit".into(), json!(limit));
    if let Some(s) = status.filter(|s| !s.trim().is_empty()) {
        body.insert("status".into(), json!(s));
    }
    client.post("campaign/GetAll", &Value::Object(body))
}

pub fn get(client: &Client, id: i64) -> Result<Value> {
    client.get(&format!("campaign/GetById?campaignId={id}"))
}

pub fn pause(client: &Client, id: i64) -> Result<Value> {
    client.post("campaign/Pause", &obj! { "campaignId" => id })
}

pub fn resume(client: &Client, id: i64) -> Result<Value> {
    client.post("campaign/Resume", &obj! { "campaignId" => id })
}

pub fn add_leads(client: &Client, id: i64, file: Option<String>) -> Result<Value> {
    let json_text = if let Some(path) = file {
        std::fs::read_to_string(&path).map_err(|e| Error::invalid(format!("Failed to read file '{path}': {e}")))?
    } else if !std::io::stdin().is_terminal() {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s).map_err(|e| Error::invalid(format!("Failed to read stdin: {e}")))?;
        s
    } else {
        return Err(Error::invalid("Provide leads via stdin or --file")
            .fix(format!("echo '[{{\"linkedinUrl\": \"...\"}}]' | heyreach campaigns add-leads {id}")));
    };

    let leads: Value =
        serde_json::from_str(&json_text).map_err(|e| Error::invalid(format!("Failed to parse JSON leads: {e}")))?;
    if !leads.is_array() {
        return Err(Error::invalid("Input must be a JSON array of lead objects"));
    }

    client.post(
        "campaign/AddLeadsToCampaign",
        &obj! {
            "campaignId" => id,
            "leads" => leads
        },
    )
}

pub fn stop_lead(client: &Client, campaign_id: i64, lead_id: i64) -> Result<Value> {
    client.post(
        "campaign/StopLeadInCampaign",
        &obj! {
            "campaignId" => campaign_id,
            "leadId" => lead_id
        },
    )
}
