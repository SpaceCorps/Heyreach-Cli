use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;

pub fn list(
    client: &Client,
    limit: u32,
    offset: u32,
    campaign_id: Option<i64>,
    sender_id: Option<i64>,
) -> Result<Value> {
    let mut body = serde_json::Map::new();
    body.insert("offset".into(), json!(offset));
    body.insert("limit".into(), json!(limit));
    if let Some(cid) = campaign_id {
        body.insert("campaignId".into(), json!(cid));
    }
    if let Some(sid) = sender_id {
        body.insert("senderId".into(), json!(sid));
    }
    client.post("conversation/GetConversationsV2", &Value::Object(body))
}

pub fn send(client: &Client, lead_id: i64, message: String, sender_id: Option<i64>) -> Result<Value> {
    let mut body = serde_json::Map::new();
    body.insert("leadId".into(), json!(lead_id));
    body.insert("message".into(), json!(message));
    if let Some(sid) = sender_id {
        body.insert("senderId".into(), json!(sid));
    }
    client.post("conversation/SendMessage", &Value::Object(body))
}
