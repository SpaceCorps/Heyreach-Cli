//! HTTP to the HeyReach API, and the translation from HTTP status to [`ErrorCode`].
//!
//! One blocking agent per process: a CLI makes a handful of requests, so an async runtime
//! would cost more in startup than it could save.

use std::time::Duration;

use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};
use crate::obj;

const DEFAULT_BASE: &str = "https://api.heyreach.io/api/public/";
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    api_key: String,
    verbose: bool,
}

enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Client {
    pub fn new(api_key: &str, verbose: bool) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(100)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("heyreach-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut base = std::env::var("HEYREACH_API_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, api_key: api_key.to_string(), verbose }
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Post, path, Some(body))
    }

    #[allow(dead_code)]
    pub fn post_empty(&self, path: &str) -> Result<Value> {
        self.send(Method::Post, path, None)
    }

    pub fn put(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Put, path, Some(body))
    }

    pub fn delete(&self, path: &str) -> Result<Value> {
        self.send(Method::Delete, path, None)
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let clean_path = path.trim_start_matches('/');
        let url = format!("{}{}", self.base, clean_path);

        if self.verbose {
            let m = match method {
                Method::Get => "GET",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Delete => "DELETE",
            };
            eprintln!(">> {m} {url}");
        }

        macro_rules! headers {
            ($req:expr) => {
                $req.header("X-API-KEY", &self.api_key).header("Accept", "application/json")
            };
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Delete, _) => headers!(self.agent.delete(&url)).call(),
            (m, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                let req = match m {
                    Method::Post => self.agent.post(&url),
                    Method::Put => self.agent.put(&url),
                    _ => unreachable!(),
                };
                headers!(req).header("Content-Type", "application/json").send(&json[..])
            }
            (m, None) => {
                let req = match m {
                    Method::Post => self.agent.post(&url),
                    Method::Put => self.agent.put(&url),
                    _ => unreachable!(),
                };
                headers!(req).send_empty()
            }
        };

        let response = result.map_err(transport_error)?;
        read(response, self.verbose)
    }
}

fn read(mut response: Response<ureq::Body>, verbose: bool) -> Result<Value> {
    let status = response.status().as_u16();
    if verbose {
        eprintln!("<< {status} {}", response.status());
    }

    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if status == 429 {
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);
        return Err(Error::new(ErrorCode::RateLimited, format!("Rate limited (429). Retry after {retry_after}s"))
            .detail(String::from_utf8_lossy(&bytes).trim().to_string())
            .fix("Back off before retrying."));
    }

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(obj! { "status" => "ok" });
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the HeyReach API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    let parsed_message = try_extract_error_message(body);

    let e = match status {
        401 | 403 => Error::new(
            ErrorCode::AuthRequired,
            parsed_message.unwrap_or_else(|| "The API key was rejected or invalid.".into()),
        )
        .fix("Check your API key: heyreach auth check (or run 'heyreach login <name>')"),
        404 => Error::new(
            ErrorCode::NotFound,
            parsed_message.unwrap_or_else(|| "The requested resource does not exist.".into()),
        ),
        429 => Error::new(ErrorCode::RateLimited, "Rate limited by the HeyReach API.").fix("Back off before retrying."),
        400 | 422 => {
            Error::new(ErrorCode::InvalidInput, parsed_message.unwrap_or_else(|| "The API refused the request.".into()))
        }
        s if s >= 500 => Error::new(
            ErrorCode::Network,
            parsed_message.unwrap_or_else(|| "The HeyReach API returned a server error.".into()),
        )
        .fix("Retry; if it persists the platform is having trouble."),
        _ => Error::new(ErrorCode::Error, parsed_message.unwrap_or_else(|| "The request failed.".into())),
    };
    e.detail(detail)
}

fn try_extract_error_message(body: &str) -> Option<String> {
    let val = serde_json::from_str::<Value>(body).ok()?;
    if let Some(msg) = val.get("message").and_then(Value::as_str) {
        return Some(msg.to_string());
    }
    if let Some(msg) = val.get("errorMessage").and_then(Value::as_str) {
        return Some(msg.to_string());
    }
    if let Some(err) = val.get("error").and_then(Value::as_str) {
        return Some(err.to_string());
    }
    if let Some(title) = val.get("title").and_then(Value::as_str) {
        let mut text = title.to_string();
        if let Some(errors) = val.get("errors").and_then(Value::as_object) {
            let mut details = Vec::new();
            for (key, val_arr) in errors {
                if let Some(arr) = val_arr.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            details.push(format!("{key}: {s}"));
                        }
                    }
                }
            }
            if !details.is_empty() {
                text.push_str(" - ");
                text.push_str(&details.join("; "));
            }
        }
        return Some(text);
    }
    None
}

/// Percent-encodes one path segment, so an id or query param can never escape its place in the URL.
#[allow(dead_code)]
pub fn seg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Builds `?a=1&b=2` from the pairs that have a value.
#[allow(dead_code)]
pub fn query(pairs: &[(&str, Option<String>)]) -> String {
    let parts: Vec<String> = pairs.iter().filter_map(|(k, v)| v.as_ref().map(|v| format!("{k}={}", seg(v)))).collect();
    if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seg_escapes_reserved() {
        assert_eq!(seg("campaign_123"), "campaign_123");
        assert_eq!(seg("John Doe"), "John%20Doe");
    }

    #[test]
    fn query_skips_missing() {
        assert_eq!(query(&[("a", None), ("b", Some("1".into()))]), "?b=1");
        assert_eq!(query(&[("a", None)]), "");
    }

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(400, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(500, "").code, ErrorCode::Network);
    }
}
