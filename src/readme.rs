//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a CLAUDE.md; `--json` gives the same rules as data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "heyreach",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call",
                "7" => "no_account - run heyreach accounts list",
            },
        });
        return;
    }
    println!("{README}");
}

/// HeyReach API version this build targets.
pub const API_VERSION: &str = "1.0.0";

const RULES: &[&str] = &[
    "Pass --account <name>, --api-key <key>, or set HEYREACH_API_KEY for API authentication.",
    "Run 'heyreach accounts list' first if you do not know which accounts exist.",
    "On code auth_required, stop and surface the remediation string. Do not retry blindly.",
    "HeyReach rate limits to 300 requests per minute. On code rate_limited (429), back off.",
    "List endpoints support --limit and --offset pagination.",
    "Campaigns add-leads accepts JSON arrays of leads via --file or standard input.",
    "Output is YAML by default for readability; use --json when piping into jq or agent loops.",
];

const README: &str = r#"# heyreach - agent operating manual

A native CLI for the HeyReach LinkedIn outreach API: campaigns, lead lists, leads, sender accounts,
conversations, stats, webhooks, and tags. Results are YAML on stdout, errors are YAML on stderr,
and `--json` switches both to JSON.

## Authentication & Accounts

You can authenticate in three ways:

1. **Named Accounts (Keystore)**:
   Store your API key securely in the native OS keystore:
   ```bash
   heyreach login [<name>] [--api-key <key>]
   heyreach accounts add <name> --api-key <key> [--force]
   printf %s "$KEY" | heyreach accounts add <name> --api-key-stdin
   heyreach accounts list [--check]
   heyreach accounts test <name>
   heyreach accounts remove <name> --yes
   ```
   When accounts are configured, pass `-a, --account <name>` to execute against that account.

2. **Environment Variable**:
   Set `HEYREACH_API_KEY`:
   ```bash
   export HEYREACH_API_KEY="your-api-key"
   heyreach auth check
   ```

3. **Direct Flag**:
   Pass `--api-key <key>` directly on any command.

## Commands Overview

### Authentication
- `heyreach auth check`: Validate the active API key with HeyReach.

### Campaigns
- `heyreach campaigns list [--limit <int>] [--offset <int>] [--status <active|paused|draft|completed>]`: List campaigns.
- `heyreach campaigns get <id>`: Get campaign details.
- `heyreach campaigns pause <id>`: Pause a campaign.
- `heyreach campaigns resume <id>`: Resume a paused campaign.
- `heyreach campaigns add-leads <id> [--file <path>]`: Add an array of leads from file or stdin.
- `heyreach campaigns stop-lead <campaign-id> --lead <lead-id>`: Stop a lead in a campaign.

### Lead Lists
- `heyreach lists list [--limit <int>] [--offset <int>]`: List lead lists.
- `heyreach lists get <id>`: Get list details.
- `heyreach lists create --name <name> [--type <user|company>]`: Create an empty list.
- `heyreach lists leads <id> [--limit <int>] [--offset <int>] [--keyword <string>]`: Get leads in list.
- `heyreach lists add-lead <id> --linkedin-url <url>`: Add a lead to a list.
- `heyreach lists remove-lead <id> --lead <lead-id>`: Remove a lead from a list.
- `heyreach lists companies <id> [--limit <int>] [--offset <int>]`: Get companies in a list.

### Leads
- `heyreach leads get --linkedin-url <url>`: Get lead by LinkedIn profile URL.
- `heyreach leads lists --linkedin-url <url>`: Get all lists containing a lead.
- `heyreach leads update-status --lead <lead-id> --status <status>`: Update lead status.

### LinkedIn Sender Accounts
- `heyreach senders list [--limit <int>] [--offset <int>]`: List connected LinkedIn senders.
- `heyreach senders get <id>`: Get sender details.
- `heyreach senders network <id> [--limit <int>] [--offset <int>]`: Get sender connections.

### Conversations
- `heyreach conversations list [--limit <int>] [--offset <int>] [--campaign-id <id>] [--sender-id <id>]`: List conversations.
- `heyreach conversations send --lead <lead-id> --message <text> [--sender-id <id>]`: Send a message.

### Analytics & Stats
- `heyreach stats get [--from <YYYY-MM-DD>] [--to <YYYY-MM-DD>] [--campaign-id <id>] [--sender-id <id>]`: Get overall metrics.

### Webhooks
- `heyreach webhooks list [--limit <int>] [--offset <int>]`: List webhooks.
- `heyreach webhooks get <id>`: Get webhook details.
- `heyreach webhooks create --name <name> --url <url> --event <event> [--campaign-id <id>]`: Create webhook.
- `heyreach webhooks update <id> [--name <name>] [--url <url>] [--event <event>] [--active <true|false>]`: Update webhook.
- `heyreach webhooks delete <id>`: Delete webhook.

### Tags
- `heyreach tags create --name <name> [--color <hex>]`: Create workspace tag.

## Exit Codes

- `0`: Success
- `1`: Error (generic/unclassified)
- `2`: Network failure (connection lost, timeout, 5xx server error)
- `3`: Auth required (missing, rejected, or invalid API key)
- `4`: Not found (requested resource does not exist)
- `5`: Rate limited (HTTP 429; retry after backing off)
- `6`: Invalid input (malformed JSON, bad arguments)
- `7`: No account specified (pass --account <name>, --api-key <key>, or set HEYREACH_API_KEY)
"#;
