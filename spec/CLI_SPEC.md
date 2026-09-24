# Heyreach CLI Specification

A CLI tool wrapping the [HeyReach API](https://documenter.getpostman.com/view/23808049/2sA2xb5F75) optimized for LLM agent consumption.

## Design Principles

- **Machine-readable by default**: All output is YAML. JSON and human-friendly tables are opt-in via `--format json|table`.
- **Predictable exit codes**: `0` = success, `1` = user/input error, `2` = API/network error.
- **Consistent flags**: Every list command supports `--limit` / `--offset`. Every mutating command echoes the resulting object.
- **No interactive prompts**: Every parameter is a flag — agents never get stuck on a prompt.
- **Env-based auth**: API key comes from `HEYREACH_API_KEY` environment variable. Can be overridden with `--api-key`.

## Authentication

| Source | Priority |
|---|---|
| `--api-key <key>` flag | 1 (highest) |
| `HEYREACH_API_KEY` env var | 2 |

Base URL: `https://api.heyreach.io/api/public`
Header: `X-API-KEY: <key>`
Rate limit: 300 req/min (CLI should surface `429` responses clearly).

## Global Options

| Flag | Description |
|---|---|
| `--api-key <key>` | Override env var |
| `--format <yaml\|json\|table>` | Output format (default: `yaml`) |
| `--no-color` | Disable colored output |
| `--verbose` | Print HTTP method, URL, status code to stderr |
| `--version` | Print version and exit |
| `--help` | Print help and exit |

## Commands

### `auth check`
Validate the API key.

```
heyreach auth check
```

**Output**:
```yaml
valid: true
message: API key is valid
```

---

### `campaigns list`
List all campaigns.

```
heyreach campaigns list [--limit 10] [--offset 0] [--status <active|paused|draft|completed>]
```

**Output**:
```yaml
total: 42
items:
  - id: 12345
    name: Q1 Outreach
    status: active
  - id: 12346
    name: Follow-ups
    status: paused
```

### `campaigns get <id>`
Get campaign details.

```
heyreach campaigns get 12345
```

### `campaigns pause <id>`
Pause a running campaign.

```
heyreach campaigns pause 12345
```

### `campaigns resume <id>`
Resume a paused campaign.

```
heyreach campaigns resume 12345
```

### `campaigns add-leads <id>`
Add leads to a campaign. Reads from stdin or `--file`.

```
echo '[{"linkedinUrl":"https://linkedin.com/in/jdoe"}]' | heyreach campaigns add-leads 12345
heyreach campaigns add-leads 12345 --file leads.json
```

**Input shape** (array of):
```json
{
  "linkedinUrl": "string (required)",
  "firstName": "string",
  "lastName": "string",
  "email": "string",
  "company": "string",
  "position": "string"
}
```

**Output**:
```yaml
added: 5
updated: 1
failed: 0
```

### `campaigns stop-lead <campaign-id> --lead <lead-id>`
Stop a specific lead in a campaign.

```
heyreach campaigns stop-lead 12345 --lead 67890
```

---

### `lists list`
List all lead lists.

```
heyreach lists list [--limit 10] [--offset 0]
```

### `lists get <id>`
Get list details.

```
heyreach lists get 12345
```

### `lists create`
Create an empty list.

```
heyreach lists create --name "Q1 Prospects" [--type user|company]
```

**Output**: created list object.

### `lists leads <id>`
Get leads from a list.

```
heyreach lists leads 12345 [--limit 50] [--offset 0] [--keyword "engineer"]
```

### `lists add-lead <id>`
Add a lead to a list.

```
heyreach lists add-lead 12345 --linkedin-url "https://linkedin.com/in/jdoe"
```

### `lists remove-lead <id>`
Remove a lead from a list.

```
heyreach lists remove-lead 12345 --lead <lead-id>
```

### `lists companies <id>`
Get companies from a company list.

```
heyreach lists companies 12345 [--limit 50] [--offset 0]
```

---

### `leads get`
Get lead details by LinkedIn URL.

```
heyreach leads get --linkedin-url "https://linkedin.com/in/jdoe"
```

### `leads lists`
Get all lists containing a lead.

```
heyreach leads lists --linkedin-url "https://linkedin.com/in/jdoe"
```

---

### `senders list`
List all LinkedIn sender accounts.

```
heyreach senders list [--limit 10] [--offset 0]
```

### `senders get <id>`
Get sender account details.

```
heyreach senders get 12345
```

### `senders network <id>`
Get network/connections for a sender account.

```
heyreach senders network 12345 [--limit 50] [--offset 0]
```

---

### `conversations list`
List conversations with filtering.

```
heyreach conversations list [--limit 20] [--offset 0] [--campaign-id <id>] [--sender-id <id>]
```

### `conversations send`
Send a message to a lead.

```
heyreach conversations send --lead <lead-id> --message "Hello!" [--sender-id <id>]
```

---

### `stats get`
Get overall stats.

```
heyreach stats get [--from 2025-01-01] [--to 2025-03-01] [--campaign-id <id>] [--sender-id <id>]
```

**Output**:
```yaml
connectionsSent: 120
connectionsAccepted: 45
messagesSent: 200
messagesReplied: 30
responseRate: 0.15
connectionRate: 0.375
```

---

### `webhooks list`
List all webhooks.

```
heyreach webhooks list [--limit 10] [--offset 0]
```

### `webhooks get <id>`
Get webhook details.

```
heyreach webhooks get 12345
```

### `webhooks create`
Create a webhook.

```
heyreach webhooks create --name "New replies" --url "https://example.com/hook" --event message_reply [--campaign-id <id>]
```

**Events**: `connection_request_sent`, `connection_request_accepted`, `message_sent`, `message_reply`, `inmail_sent`, `inmail_reply`, `follow_sent`, `profile_viewed`, `campaign_completed`, `tag_updated`

### `webhooks update <id>`
Update a webhook.

```
heyreach webhooks update 12345 [--name "..."] [--url "..."] [--event ...] [--active true|false]
```

### `webhooks delete <id>`
Delete a webhook.

```
heyreach webhooks delete 12345
```

---

### `tags create`
Create workspace tags.

```
heyreach tags create --name "Hot Lead" [--color "#FF0000"]
```

---

## LLM Agent Usage Patterns

### Why YAML-first?
YAML is more token-efficient than JSON (no braces, quotes, or commas), which matters for LLM context windows. Agents parse YAML reliably. The CLI returns YAML to stdout. Errors go to stderr as YAML:
```yaml
error: "message"
code: 1
```

### Piping & Composition
```bash
# Get all campaign IDs
heyreach campaigns list | yq '.items[].id'

# Add leads from a CSV-converted JSON file
cat leads.json | heyreach campaigns add-leads 12345

# Get stats for a specific campaign and extract response rate
heyreach stats get --campaign-id 12345 | yq '.responseRate'

# Use JSON output for jq pipelines
heyreach campaigns list --format json | jq '.items[].id'
```

### Error Handling for Agents
```
# Missing API key → exit 1
error: HEYREACH_API_KEY not set
code: 1

# Invalid ID → exit 1
error: Campaign not found
code: 1

# Rate limited → exit 2
error: "Rate limited (429). Retry after 60s"
code: 2

# Network failure → exit 2
error: Connection failed
code: 2
```

## Project Structure

```
src/Heyreach.Console/
├── Program.cs                     # CommandApp setup
├── Infrastructure/
│   ├── HeyreachApiClient.cs       # Typed HTTP client
│   ├── OutputHelper.cs            # YAML/JSON/table output
│   └── GlobalSettings.cs          # --api-key, --format, etc.
├── Commands/
│   ├── Auth/
│   │   └── CheckCommand.cs
│   ├── Campaigns/
│   │   ├── ListCommand.cs
│   │   ├── GetCommand.cs
│   │   ├── PauseCommand.cs
│   │   ├── ResumeCommand.cs
│   │   ├── AddLeadsCommand.cs
│   │   └── StopLeadCommand.cs
│   ├── Lists/
│   │   ├── ListCommand.cs
│   │   ├── GetCommand.cs
│   │   ├── CreateCommand.cs
│   │   ├── LeadsCommand.cs
│   │   ├── AddLeadCommand.cs
│   │   ├── RemoveLeadCommand.cs
│   │   └── CompaniesCommand.cs
│   ├── Leads/
│   │   ├── GetCommand.cs
│   │   └── ListsCommand.cs
│   ├── Senders/
│   │   ├── ListCommand.cs
│   │   ├── GetCommand.cs
│   │   └── NetworkCommand.cs
│   ├── Conversations/
│   │   ├── ListCommand.cs
│   │   └── SendCommand.cs
│   ├── Stats/
│   │   └── GetCommand.cs
│   ├── Webhooks/
│   │   ├── ListCommand.cs
│   │   ├── GetCommand.cs
│   │   ├── CreateCommand.cs
│   │   ├── UpdateCommand.cs
│   │   └── DeleteCommand.cs
│   └── Tags/
│       └── CreateCommand.cs
```
