use serde_json::Value;

use crate::client::Client;
use crate::error::Result;
use crate::obj;

pub fn list(client: &Client, limit: u32, offset: u32) -> Result<Value> {
    client.post(
        "linkedInAccount/GetAll",
        &obj! {
            "offset" => offset,
            "limit" => limit
        },
    )
}

pub fn get(client: &Client, id: i64) -> Result<Value> {
    client.get(&format!("linkedInAccount/GetById?senderId={id}"))
}

pub fn network(client: &Client, id: i64, limit: u32, offset: u32) -> Result<Value> {
    client.post(
        "myNetwork/GetMyNetworkForSender",
        &obj! {
            "senderId" => id,
            "offset" => offset,
            "limit" => limit
        },
    )
}
