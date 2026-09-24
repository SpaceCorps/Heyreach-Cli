use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::obj;

pub fn list(client: &Client, limit: u32, offset: u32) -> Result<Value> {
    client.post(
        "list/GetAll",
        &obj! {
            "offset" => offset,
            "limit" => limit
        },
    )
}

pub fn get(client: &Client, id: i64) -> Result<Value> {
    client.get(&format!("list/GetById?listId={id}"))
}

pub fn create(client: &Client, name: String, r#type: String) -> Result<Value> {
    let list_type = match r#type.to_lowercase().as_str() {
        "company" => "COMPANY_LIST",
        _ => "USER_LIST",
    };
    client.post(
        "list/CreateEmptyList",
        &obj! {
            "name" => name,
            "type" => list_type
        },
    )
}

pub fn leads(client: &Client, id: i64, limit: u32, offset: u32, keyword: Option<String>) -> Result<Value> {
    let mut body = serde_json::Map::new();
    body.insert("listId".into(), json!(id));
    body.insert("offset".into(), json!(offset));
    body.insert("limit".into(), json!(limit));
    if let Some(kw) = keyword.filter(|k| !k.trim().is_empty()) {
        body.insert("keyword".into(), json!(kw));
    }
    client.post("list/GetLeadsFromList", &Value::Object(body))
}

pub fn add_lead(client: &Client, id: i64, linkedin_url: String) -> Result<Value> {
    client.post(
        "list/AddLeadsToListV2",
        &obj! {
            "listId" => id,
            "leads" => json!([{"linkedinUrl": linkedin_url}])
        },
    )
}

pub fn remove_lead(client: &Client, id: i64, lead_id: i64) -> Result<Value> {
    client.post(
        "list/DeleteLeadFromList",
        &obj! {
            "listId" => id,
            "leadId" => lead_id
        },
    )
}

pub fn companies(client: &Client, id: i64, limit: u32, offset: u32) -> Result<Value> {
    client.post(
        "list/GetCompaniesFromList",
        &obj! {
            "listId" => id,
            "offset" => offset,
            "limit" => limit
        },
    )
}
