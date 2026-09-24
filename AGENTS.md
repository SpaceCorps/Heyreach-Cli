# AGENTS.md

Notes for whoever extends this next.

`heyreach` is a Rust CLI over the HeyReach Public REST API, built to be driven by humans and autonomous LLM agents. It replaced a .NET global tool of the same name and keeps its interface: the same commands and flags, YAML-first output, structured error envelopes and exit codes, and adds secure native keystore integration and multi-account isolation.

For the manual the *agent* reads, run `heyreach agent-readme` — that text lives in `src/readme.rs` and is the tool's actual interface for its main audience. This file is for the human editing the source.

## Commands

```bash
cargo build --release              # target/release/heyreach
cargo test                         # unit tests + tests/cli.rs against mock API
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # put it on PATH
```

Use a throwaway config directory when testing so you never touch real credentials:

```bash
export HEYREACH_CONFIG_DIR=$(mktemp -d) HEYREACH_SECRET_STORE=plaintext HEYREACH_ALLOW_PLAINTEXT_STORE=1
```

| Variable | Effect |
| --- | --- |
| `HEYREACH_CONFIG_DIR` | Overrides the config/secrets directory location |
| `HEYREACH_SECRET_STORE` | Forces a keystore backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `HEYREACH_ALLOW_PLAINTEXT_STORE=1` | Permits plaintext fallback where no keystore exists |
| `HEYREACH_API_URL` | Overrides the API base URL — how `tests/cli.rs` points at its mock |
| `HEYREACH_API_KEY` | Optional environment variable for direct API key authentication |

## Layout

```
src/
  main.rs          arg parsing, the --json pre-scan, clap errors -> invalid_input envelopes
  cli.rs           the whole command tree (clap derive); help text lives here
  commands/
    mod.rs         dispatch, and shared print helper
    accounts.rs    accounts add|list|test|remove
    login.rs       login command (browser/interactive or token prompt)
    auth.rs        auth check
    campaigns.rs   campaigns list|get|pause|resume|add-leads|stop-lead
    lists.rs       lists list|get|create|leads|add-lead|remove-lead|companies
    leads.rs       leads get|lists|update-status
    senders.rs     senders list|get|network
    conversations.rs conversations list|send
    stats.rs       stats get
    webhooks.rs    webhooks list|get|create|update|delete
    tags.rs        tags create
  client.rs        blocking HTTP (ureq + rustls), status -> ErrorCode, ASP.NET problem details parser
  error.rs         ErrorCode (= exit code) and Error {message, detail, remediation}
  output.rs        YAML by default, JSON with --json, the error envelope, obj! macro
  account.rs       --account / --api-key resolution -> Resolved (name, config, key)
  config.rs        config.yaml, paths, atomic writes, 0600, cross-process lock
  secrets.rs       Keychain (security), libsecret (secret-tool), DPAPI, plaintext
  readme.rs        agent-readme text, rules, and exit code mappings
tests/cli.rs       drives the binary against an in-process TCP mock of the API
```

## Why it is built this way

**Blocking HTTP, no async runtime.** A CLI makes one to a few requests. Tokio would cost more in startup than it saves; `accounts list --check` fans out with scoped threads (`std::thread::scope`).

**The Keychain goes through `/usr/bin/security`, not the Security framework.** Items created by command-line tools list `security` as a trusted app, avoiding code-signing prompt loops on rebuilt binaries.

**Responses stay `serde_json::Value`.** The upstream API adds fields without notice. Typed structs would drop new fields or fail on new enum values. Request bodies are built as `Value` too with `obj!`, keeping key order stable.

## How a command is wired

1. Add the variant to the subcommand in `src/cli.rs`.
2. Implement the API interaction in `src/commands/<domain>.rs`.
3. Dispatch in `src/commands/mod.rs`.
4. Return `Result<Value>` or `Result<()>` and output using `print(...)`.
5. Add test coverage in `tests/cli.rs`.

## Invariants

- Secrets never touch `config.yaml`. Config holds account metadata; secrets live in the OS keystore.
- All errors print structured envelopes to `stderr` with stable exit codes matching `ErrorCode`.
- When called with `--json` (or `--format json`), outputs parseable JSON to stdout.
