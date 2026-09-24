use serde_json::Value;

use crate::client::Client;
use crate::error::{Error, Result};
use crate::obj;

pub fn get(client: &Client, linkedin_url: String) -> Result<Value> {
    client.post(
        "lead/GetLead",
        &obj! {
            "profileUrl" => linkedin_url
        },
    )
}

pub fn lists(client: &Client, linkedin_url: String) -> Result<Value> {
    client.post(
        "list/GetListsForLead",
        &obj! {
            "profileUrl" => linkedin_url
        },
    )
}

pub fn update_status(client: &Client, lead_id: i64, status: String) -> Result<Value> {
    let valid = ["pending", "contacted", "replied", "connected", "not_interested", "bounced"];
    let lower = status.to_lowercase();
    if !valid.contains(&lower.as_str()) {
        return Err(Error::invalid(format!("Invalid status '{status}'. Must be one of: {}", valid.join(", "))));
    }
    client.post(
        "lead/UpdateStatus",
        &obj! {
            "leadId" => lead_id,
            "status" => status
        },
    )
}
