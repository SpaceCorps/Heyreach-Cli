use serde_json::Value;

use crate::client::Client;
use crate::error::Result;
use crate::obj;

pub fn check(client: &Client) -> Result<Value> {
    client.get("auth/CheckApiKey")?;
    Ok(obj! {
        "valid" => true,
        "message" => "API key is valid"
    })
}
