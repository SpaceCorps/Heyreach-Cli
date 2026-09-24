# HeyReach CLI

[![Release](https://img.shields.io/github/v/release/SpaceCorps/Heyreach-Cli?color=blue&label=version)](https://github.com/SpaceCorps/Heyreach-Cli/releases/latest)
[![CI](https://github.com/SpaceCorps/Heyreach-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Heyreach-Cli/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online-success)](https://spacecorps.github.io/Heyreach-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A high-performance, native command-line tool and autonomous agent interface for the [HeyReach](https://heyreach.io) LinkedIn outreach API. Built in native Rust 2024 for developers, growth engineering teams, and autonomous AI agents.

---

## Highlights

- ⚡ **Sub-3ms Startup**: Compiled as a native static binary with zero runtime dependencies. Cold starts in 1–3 ms.
- 🔐 **OS Keystore Integration**: `heyreach login` and `heyreach accounts add` store API keys securely in native OS vaults (macOS Keychain, Windows DPAPI, Linux Secret Service).
- 🌐 **Comprehensive HeyReach API Surface**: Full coverage across campaigns, lead lists, leads, sender accounts, conversations, stats, webhooks, and tags.
- 🤖 **AI Agent Native**: Machine-readable `--json` output, YAML default for terminal readability, structured error envelopes on stderr, and built-in `heyreach agent-readme [--json]`.
- 🏢 **Multi-Account Safety**: Strict account scoping with `-a, --account <name>` protects agency multi-client setups from accidental cross-tenant execution.

---

## Installation

### Using Cargo

```bash
cargo install --git https://github.com/SpaceCorps/Heyreach-Cli --locked
```

### Pre-built Standalone Binaries

Download standalone binary archives directly from the [GitHub Releases](https://github.com/SpaceCorps/Heyreach-Cli/releases/latest) page:

| Platform | Architecture | Binary Package |
|:---|:---|:---|
| **macOS** | Apple Silicon (`aarch64`) | `heyreach-v1.0.0-aarch64-apple-darwin.tar.gz` |
| **macOS** | Intel (`x86_64`) | `heyreach-v1.0.0-x86_64-apple-darwin.tar.gz` |
| **Linux** | x86_64 (musl static) | `heyreach-v1.0.0-x86_64-unknown-linux-musl.tar.gz` |
| **Linux** | ARM64 (musl static) | `heyreach-v1.0.0-aarch64-unknown-linux-musl.tar.gz` |
| **Windows**| x64 (MSVC) | `heyreach-v1.0.0-x86_64-pc-windows-msvc.zip` |

---

## Quickstart

### 1. Authenticate

Run `heyreach login` to open your browser to the HeyReach API settings, verify the key against `GET /auth/CheckApiKey`, and store it in your OS keystore:

```bash
# Interactive browser login (saved under account 'default')
heyreach login

# Log in with a specific account name
heyreach login prod

# Headless / CI pipeline login (reads key from stdin with no shell history trace)
echo "$HEYREACH_API_KEY" | heyreach login ci --api-key-stdin
```

Or authenticate directly via environment variable or flag:
```bash
export HEYREACH_API_KEY="your-api-key"
heyreach auth check
```

### 2. Manage Campaigns

```bash
# List all active campaigns
heyreach campaigns list --status active

# Pause or resume a campaign
heyreach campaigns pause 12345
heyreach campaigns resume 12345

# Ingest leads in bulk from JSON via stdin
cat leads.json | heyreach campaigns add-leads 12345
```

### 3. Lead Lists & Contacts

```bash
# Create a new lead list
heyreach lists create --name "Q1 Prospects" --type user

# Add a lead to a list
heyreach lists add-lead 12345 --linkedin-url "https://linkedin.com/in/target"

# Query lead details
heyreach leads get --linkedin-url "https://linkedin.com/in/target"
```

---

## Command Reference

### Authentication & Accounts

| Command | Description |
|:---|:---|
| `heyreach auth check` | Validate active API key validity |
| `heyreach login [name]` | Authenticate interactively via browser and save to OS keystore |
| `heyreach accounts list [--check]` | List configured accounts (pass `--check` to verify key validity) |
| `heyreach accounts test <name>` | Test credentials for a specific account |
| `heyreach accounts add <name>` | Add an account and API key to keystore |
| `heyreach accounts remove <name> [--yes]` | Remove account and purge key from keystore |

### Campaigns

| Command | Description |
|:---|:---|
| `heyreach campaigns list` | List outreach campaigns (`--limit`, `--offset`, `--status`) |
| `heyreach campaigns get <id>` | Retrieve campaign details |
| `heyreach campaigns pause <id>` | Pause an active campaign |
| `heyreach campaigns resume <id>` | Resume a paused campaign |
| `heyreach campaigns add-leads <id>` | Ingest leads from file (`--file`) or stdin |
| `heyreach campaigns stop-lead <cid> --lead <lid>` | Stop a specific lead in a campaign |

### Lead Lists & Leads

| Command | Description |
|:---|:---|
| `heyreach lists list` | List lead lists (`--limit`, `--offset`) |
| `heyreach lists get <id>` | Retrieve list details |
| `heyreach lists create` | Create an empty list (`--name`, `--type user\|company`) |
| `heyreach lists leads <id>` | Query leads in list (`--limit`, `--offset`, `--keyword`) |
| `heyreach lists add-lead <id>` | Add lead to list (`--linkedin-url`) |
| `heyreach lists remove-lead <id>` | Remove lead from list (`--lead <id>`) |
| `heyreach lists companies <id>` | Query companies from company list |
| `heyreach leads get` | Fetch lead by LinkedIn profile URL (`--linkedin-url`) |
| `heyreach leads lists` | Query all lists containing a lead |
| `heyreach leads update-status` | Update lead status (`--lead`, `--status`) |

### Senders & Conversations

| Command | Description |
|:---|:---|
| `heyreach senders list` | List connected LinkedIn sender accounts |
| `heyreach senders get <id>` | Fetch sender account details |
| `heyreach senders network <id>` | Query network connections for a sender account |
| `heyreach conversations list` | Query conversations (`--campaign-id`, `--sender-id`) |
| `heyreach conversations send` | Send a message to a lead (`--lead`, `--message`, `--sender-id`) |

### Analytics, Webhooks & Tags

| Command | Description |
|:---|:---|
| `heyreach stats get` | Query outreach analytics (`--from`, `--to`, `--campaign-id`, `--sender-id`) |
| `heyreach webhooks list` | List registered webhooks |
| `heyreach webhooks get <id>` | Retrieve webhook details |
| `heyreach webhooks create` | Create a webhook (`--name`, `--url`, `--event`, `--campaign-id`) |
| `heyreach webhooks update <id>` | Update webhook configuration |
| `heyreach webhooks delete <id>` | Delete a webhook |
| `heyreach tags create` | Create a workspace tag (`--name`, `--color`) |

---

## Agentic Integration & Discovery

HeyReach CLI includes native discovery interfaces for autonomous agents and LLM toolchains:

- **Built-in Agent Manual**: `heyreach agent-readme [--json]`
- **Online Agent Manifest**: [llms.txt](https://spacecorps.github.io/Heyreach-Cli/llms.txt)
- **Comprehensive Technical Reference**: [llms-full.txt](https://spacecorps.github.io/Heyreach-Cli/llms-full.txt)
- **A2A Discovery Card**: [agent-card.json](https://spacecorps.github.io/Heyreach-Cli/.well-known/agent-card.json)
- **AgentSkills Manifest**: [agent-skills/index.json](https://spacecorps.github.io/Heyreach-Cli/.well-known/agent-skills/index.json)

### Exit Codes & Machine-Readable Errors

When commands fail, structured error envelopes are written to `stderr` in YAML (or JSON with `--json`):

```json
{
  "error": "The API key was rejected or invalid.",
  "code": "auth_required",
  "detail": "HTTP 401: Unauthorized",
  "remediation": "heyreach auth check (or run 'heyreach login <name>')"
}
```

| Exit Code | Name | Description |
|:---|:---|:---|
| `0` | `ok` | Success |
| `1` | `error` | General / unclassified error |
| `2` | `network` | Connection failure or upstream 5xx error |
| `3` | `auth_required` | Invalid, rejected, or missing API key |
| `4` | `not_found` | Resource ID not found |
| `5` | `rate_limited` | HeyReach rate limit reached (300 req/min) |
| `6` | `invalid_input` | Malformed JSON or invalid argument |
| `7` | `no_account` | No account specified when multiple exist |

---

## License & Credits

- Licensed under the [MIT License](LICENSE).
- Ported and maintained by [SpaceCorps](https://github.com/SpaceCorps).
- Original C# prototype created by [Niels Bosma](https://github.com/nielsbosma/Heyreach.Console).
