use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::obj;

pub fn list(client: &Client, limit: u32, offset: u32) -> Result<Value> {
    client.post(
        "webhook/GetAll",
        &obj! {
            "offset" => offset,
            "limit" => limit
        },
    )
}

pub fn get(client: &Client, id: i64) -> Result<Value> {
    client.get(&format!("webhook/GetById?webhookId={id}"))
}

pub fn create(client: &Client, name: String, url: String, event: String, campaign_id: Option<i64>) -> Result<Value> {
    let mut body = serde_json::Map::new();
    body.insert("name".into(), json!(name));
    body.insert("url".into(), json!(url));
    body.insert("eventType".into(), json!(event));
    if let Some(cid) = campaign_id {
        body.insert("campaignId".into(), json!(cid));
    }
    client.post("webhook/Create", &Value::Object(body))
}

pub fn update(
    client: &Client,
    id: i64,
    name: Option<String>,
    url: Option<String>,
    event: Option<String>,
    active: Option<bool>,
) -> Result<Value> {
    let mut body = serde_json::Map::new();
    body.insert("webhookId".into(), json!(id));
    if let Some(n) = name {
        body.insert("name".into(), json!(n));
    }
    if let Some(u) = url {
        body.insert("url".into(), json!(u));
    }
    if let Some(e) = event {
        body.insert("eventType".into(), json!(e));
    }
    if let Some(a) = active {
        body.insert("isActive".into(), json!(a));
    }
    client.put("webhook/Update", &Value::Object(body))
}

pub fn delete(client: &Client, id: i64) -> Result<Value> {
    let res = client.delete(&format!("webhook/Delete?webhookId={id}"))?;
    if res.as_object().is_some_and(|o| o.is_empty()) { Ok(obj! { "status" => "ok", "id" => id }) } else { Ok(res) }
}
