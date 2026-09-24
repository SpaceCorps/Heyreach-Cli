//! Drives the built binary against an in-process mock of the HeyReach API. Every test gets its
//! own config directory and the plaintext store, so nothing touches a real keystore or account.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    /// Routes are (method, path-with-query, status, body). Unmatched requests get a 404.
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}/api/public/");
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let raw_path = parts.next().unwrap_or("").to_string();
                    let path = raw_path.trim_start_matches("/api/public/").trim_start_matches('/').to_string();

                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap());
                    log.lock().unwrap().push(Recorded { method: method.clone(), path: path.clone(), headers, body });

                    let (status, resp) = routes
                        .iter()
                        .find(|(m, p, _, _)| *m == method && *p == path)
                        .map(|(_, _, s, b)| (*s, b.clone()))
                        .unwrap_or((404, json!({"code": "not_found", "message": "no route"})));
                    let text = if status == 204 { String::new() } else { resp.to_string() };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "heyreach-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_heyreach"))
            .args(args)
            .env("HEYREACH_CONFIG_DIR", &self.dir)
            .env("HEYREACH_SECRET_STORE", "plaintext")
            .env("HEYREACH_ALLOW_PLAINTEXT_STORE", "1")
            .env("HEYREACH_API_URL", &self.api)
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    fn with_account(self, name: &str, key: &str) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", name, "--api-key", key]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn auth_check_route() -> Route {
    ("GET", "auth/CheckApiKey", 200, json!({"valid": true, "message": "API key is valid"}))
}

#[test]
fn test_agent_readme() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(out["tool"], "heyreach");
    assert_eq!(out["apiVersion"], "1.0.0");
    assert!(out["rules"].is_array());
    assert_eq!(out["exitCodes"]["0"], "ok");

    let out_raw = env.run(&["agent-readme"]);
    assert_eq!(out_raw.status.code().unwrap(), 0);
    let stdout = String::from_utf8_lossy(&out_raw.stdout);
    assert!(stdout.contains("# heyreach - agent operating manual"));
}

#[test]
fn test_auth_check() {
    let mock = Mock::start(vec![auth_check_route()]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["auth", "check", "--api-key", "test_key_123"]);
    assert_eq!(code, 0);
    assert_eq!(out["valid"], true);
    assert_eq!(out["message"], "API key is valid");

    let req = mock.last("GET");
    assert_eq!(req.headers.iter().find(|(k, _)| k == "x-api-key").unwrap().1, "test_key_123");
}

#[test]
fn test_accounts_lifecycle() {
    let mock = Mock::start(vec![auth_check_route()]);
    let env = Env::new(&mock).with_account("prod", "hey_prod_key");

    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["name"], "prod");
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");

    let (code, out, _) = env.json(&["accounts", "list", "--check"]);
    assert_eq!(code, 0);
    assert_eq!(out["accounts"][0]["keyStatus"], "valid");

    let (code, out, _) = env.json(&["accounts", "test", "prod"]);
    assert_eq!(code, 0);
    assert_eq!(out["keyStatus"], "valid");

    // Remove without --yes
    let (code, _, err) = env.json(&["accounts", "remove", "prod"]);
    assert_eq!(code, 6);
    assert!(err["remediation"].as_str().unwrap().contains("--yes"));

    // Remove with --yes
    let (code, out, _) = env.json(&["accounts", "remove", "prod", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    let (_, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(out["count"], 0);
}

#[test]
fn test_campaigns_commands() {
    let mock = Mock::start(vec![
        (
            "POST",
            "campaign/GetAll",
            200,
            json!({"total": 1, "items": [{"id": 100, "name": "Q1 Outreach", "status": "active"}]}),
        ),
        ("GET", "campaign/GetById?campaignId=100", 200, json!({"id": 100, "name": "Q1 Outreach", "status": "active"})),
        ("POST", "campaign/Pause", 200, json!({"success": true, "status": "paused"})),
        ("POST", "campaign/Resume", 200, json!({"success": true, "status": "active"})),
        ("POST", "campaign/AddLeadsToCampaign", 200, json!({"added": 2, "updated": 0, "failed": 0})),
        ("POST", "campaign/StopLeadInCampaign", 200, json!({"success": true})),
    ]);
    let env = Env::new(&mock);

    // List
    let (code, out, _) =
        env.json(&["campaigns", "list", "--api-key", "key", "--limit", "5", "--offset", "0", "--status", "active"]);
    assert_eq!(code, 0);
    assert_eq!(out["total"], 1);
    let req = mock.last("POST");
    assert_eq!(req.body.as_ref().unwrap()["limit"], 5);
    assert_eq!(req.body.as_ref().unwrap()["status"], "active");

    // Get
    let (code, out, _) = env.json(&["campaigns", "get", "100", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["id"], 100);

    // Pause
    let (code, out, _) = env.json(&["campaigns", "pause", "100", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "paused");

    // Resume
    let (code, out, _) = env.json(&["campaigns", "resume", "100", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "active");

    // Add leads with file
    let temp_leads = std::env::temp_dir().join(format!("leads-{}.json", std::process::id()));
    std::fs::write(&temp_leads, r#"[{"linkedinUrl": "https://linkedin.com/in/testuser"}]"#).unwrap();
    let (code, out, _) =
        env.json(&["campaigns", "add-leads", "100", "--file", temp_leads.to_str().unwrap(), "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["added"], 2);
    let _ = std::fs::remove_file(temp_leads);

    // Stop lead
    let (code, out, _) = env.json(&["campaigns", "stop-lead", "100", "--lead", "555", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["success"], true);
}

#[test]
fn test_lists_commands() {
    let mock = Mock::start(vec![
        ("POST", "list/GetAll", 200, json!({"items": [{"id": 200, "name": "Prospects"}]})),
        ("GET", "list/GetById?listId=200", 200, json!({"id": 200, "name": "Prospects"})),
        ("POST", "list/CreateEmptyList", 200, json!({"id": 201, "name": "Engineers", "type": "USER_LIST"})),
        ("POST", "list/GetLeadsFromList", 200, json!({"items": [{"id": 1, "name": "Alice"}]})),
        ("POST", "list/AddLeadsToListV2", 200, json!({"added": 1})),
        ("POST", "list/DeleteLeadFromList", 200, json!({"deleted": 1})),
        ("POST", "list/GetCompaniesFromList", 200, json!({"items": [{"id": 10, "name": "Acme"}]})),
    ]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["lists", "list", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["items"][0]["id"], 200);

    let (code, out, _) = env.json(&["lists", "get", "200", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "Prospects");

    let (code, out, _) = env.json(&["lists", "create", "--name", "Engineers", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["type"], "USER_LIST");

    let (code, out, _) = env.json(&["lists", "leads", "200", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["items"][0]["name"], "Alice");

    let (code, out, _) =
        env.json(&["lists", "add-lead", "200", "--linkedin-url", "https://linkedin.com/in/alice", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["added"], 1);

    let (code, out, _) = env.json(&["lists", "remove-lead", "200", "--lead", "1", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["deleted"], 1);

    let (code, out, _) = env.json(&["lists", "companies", "200", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["items"][0]["name"], "Acme");
}

#[test]
fn test_leads_commands() {
    let mock = Mock::start(vec![
        (
            "POST",
            "lead/GetLead",
            200,
            json!({"id": 1, "firstName": "Alice", "profileUrl": "https://linkedin.com/in/alice"}),
        ),
        ("POST", "list/GetListsForLead", 200, json!({"items": [{"id": 200, "name": "Prospects"}]})),
        ("POST", "lead/UpdateStatus", 200, json!({"success": true, "status": "contacted"})),
    ]);
    let env = Env::new(&mock);

    let (code, out, _) =
        env.json(&["leads", "get", "--linkedin-url", "https://linkedin.com/in/alice", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["firstName"], "Alice");

    let (code, out, _) =
        env.json(&["leads", "lists", "--linkedin-url", "https://linkedin.com/in/alice", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["items"][0]["name"], "Prospects");

    let (code, out, _) =
        env.json(&["leads", "update-status", "--lead", "1", "--status", "contacted", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "contacted");

    // Invalid status validation
    let (code, _, err) =
        env.json(&["leads", "update-status", "--lead", "1", "--status", "unknown_status", "--api-key", "key"]);
    assert_eq!(code, 6);
    assert!(err["error"].as_str().unwrap().contains("Invalid status 'unknown_status'"));
}

#[test]
fn test_webhooks_and_tags() {
    let mock = Mock::start(vec![
        ("POST", "webhook/GetAll", 200, json!({"items": [{"id": 50, "name": "Hook1"}]})),
        (
            "GET",
            "webhook/GetById?webhookId=50",
            200,
            json!({"id": 50, "name": "Hook1", "url": "https://example.com/h"}),
        ),
        ("POST", "webhook/Create", 200, json!({"id": 51, "name": "Hook2"})),
        ("PUT", "webhook/Update", 200, json!({"id": 50, "name": "HookRenamed"})),
        ("DELETE", "webhook/Delete?webhookId=50", 200, json!({})),
        ("POST", "tag/Create", 200, json!({"id": 1, "displayName": "VIP"})),
    ]);
    let env = Env::new(&mock);

    let (code, out, _) = env.json(&["webhooks", "list", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["items"][0]["id"], 50);

    let (code, out, _) = env.json(&["webhooks", "get", "50", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "Hook1");

    let (code, out, _) = env.json(&[
        "webhooks",
        "create",
        "--name",
        "Hook2",
        "--url",
        "https://example.com/h2",
        "--event",
        "message_reply",
        "--api-key",
        "key",
    ]);
    assert_eq!(code, 0);
    assert_eq!(out["id"], 51);

    let (code, out, _) = env.json(&["webhooks", "update", "50", "--name", "HookRenamed", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "HookRenamed");

    let (code, out, _) = env.json(&["webhooks", "delete", "50", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "ok");

    let (code, out, _) = env.json(&["tags", "create", "--name", "VIP", "--api-key", "key"]);
    assert_eq!(code, 0);
    assert_eq!(out["displayName"], "VIP");
}

#[test]
fn test_error_status_mappings() {
    let mock = Mock::start(vec![
        ("GET", "auth/CheckApiKey", 401, json!({"message": "Invalid API key"})),
        ("GET", "campaign/GetById?campaignId=999", 404, json!({"message": "Campaign not found"})),
        ("POST", "campaign/GetAll", 429, json!({"message": "Too many requests"})),
        ("POST", "campaign/Pause", 500, json!({"message": "Internal server error"})),
    ]);
    let env = Env::new(&mock);

    // 401 -> 3 (AuthRequired)
    let (code, _, err) = env.json(&["auth", "check", "--api-key", "bad_key"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");

    // 404 -> 4 (NotFound)
    let (code, _, err) = env.json(&["campaigns", "get", "999", "--api-key", "key"]);
    assert_eq!(code, 4);
    assert_eq!(err["code"], "not_found");

    // 429 -> 5 (RateLimited)
    let (code, _, err) = env.json(&["campaigns", "list", "--api-key", "key"]);
    assert_eq!(code, 5);
    assert_eq!(err["code"], "rate_limited");

    // 500 -> 2 (Network)
    let (code, _, err) = env.json(&["campaigns", "pause", "1", "--api-key", "key"]);
    assert_eq!(code, 2);
    assert_eq!(err["code"], "network");
}
