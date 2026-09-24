use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;

pub fn create(client: &Client, name: String, color: Option<String>) -> Result<Value> {
    let mut body = serde_json::Map::new();
    body.insert("displayName".into(), json!(name));
    if let Some(c) = color {
        body.insert("color".into(), json!(c));
    }
    client.post("tag/Create", &Value::Object(body))
}
