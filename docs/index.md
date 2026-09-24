---
title: "HeyReach CLI"
description: "A high-performance native command-line tool and agent interface for the HeyReach LinkedIn outreach API. Built in Rust 2024 for developers and autonomous AI agents."
author: "SpaceCorps"
date: "2026-09-24"
canonical: "https://spacecorps.github.io/Heyreach-Cli/index.md"
---

# HeyReach CLI

A high-performance native command-line tool and autonomous agent interface for the HeyReach LinkedIn outreach API. Built in native Rust 2024 for developers, growth engineers, and autonomous AI agents.

## Quickstart

```bash
# Authenticate via browser or paste API key
heyreach login

# Or pass via environment variable
export HEYREACH_API_KEY="your-api-key"
heyreach auth check
```

## Features

- **Blazing Fast Native Rust**: Sub-3ms cold start with zero runtime dependencies.
- **AI Agent Native**: Structured JSON output (`--json`), YAML by default, and predictable exit codes.
- **Secure Keystore Integration**: Token storage in native macOS Keychain, Windows DPAPI, and Linux Secret Service.
- **Multi-Account Outreach**: Isolate agency clients and outreach accounts with `-a, --account`.
- **Campaign & Lead Orchestration**: Bulk ingest leads, manage sequences, monitor senders, and receive webhook events.

## When to Use This CLI

Use the `heyreach` CLI whenever you need to:
- Orchestrate, inspect, pause, and resume LinkedIn outreach campaigns.
- Bulk ingest leads from JSON pipelines into active campaigns.
- Manage lead lists and query enrichment data.
- Monitor sender accounts and network connections.
- Dispatch messages and manage conversation threads.
- Query conversion stats and configure real-time webhooks.

## Documentation Links

- [llms.txt](https://spacecorps.github.io/Heyreach-Cli/llms.txt)
- [Full Agent Manual](https://spacecorps.github.io/Heyreach-Cli/llms-full.txt)
- [Pricing](https://spacecorps.github.io/Heyreach-Cli/pricing.md)
- [Authentication Guide](https://spacecorps.github.io/Heyreach-Cli/auth.md)
- [GitHub Repository](https://github.com/SpaceCorps/Heyreach-Cli)
