---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the HeyReach CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for HeyReach CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the HeyReach CLI.

## Overview
The HeyReach CLI interfaces directly with the HeyReach Public REST API. Authentication is API-key-based, using the `X-API-KEY` header. Keys can be stored in the host operating system's native keychain, supplied via environment variables, or passed directly on the command line.

## Prerequisites
- A HeyReach account ([heyreach.io](https://heyreach.io))
- A valid API key generated from the HeyReach settings page (`https://app.heyreach.io/settings/api`)
- HeyReach CLI installed on your machine (`cargo install --git https://github.com/SpaceCorps/Heyreach-Cli --locked`)

## Authentication Flow

### Interactive Browser Login (`heyreach login`)
The recommended flow for local developer machines:
```bash
heyreach login [account_name]
```
1. The CLI launches your system browser to `https://app.heyreach.io/settings/api`.
2. You copy or generate your personal or team API key.
3. Paste the key into the CLI prompt (input characters are masked).
4. The CLI validates the key with a live request to `GET /auth/CheckApiKey`.
5. Upon confirmation, the key is securely saved to the native OS keyring under the account name (defaults to `default`).

### Non-Interactive / Headless Setup
For headless CI/CD environments, Docker containers, or autonomous agent runners:
```bash
echo "$HEYREACH_API_KEY" | heyreach login [account_name] --api-key-stdin
```
Or pass the key directly as a CLI flag:
```bash
heyreach accounts add [account_name] --api-key "$HEYREACH_API_KEY"
```

## Environment Variables
The CLI checks the environment for credentials when no keychain account is specified:
- `HEYREACH_API_KEY`: API key used if no keystore account is explicitly selected.
- `HEYREACH_CONFIG_DIR`: Custom directory path for `config.yaml` and local state.
- `HEYREACH_SECRET_STORE`: Forces a specific keystore backend (`keychain`, `dpapi`, `libsecret`, `plaintext`).
- `HEYREACH_ALLOW_PLAINTEXT_STORE=1`: Permits fallback plaintext storage in environments without a system keyring.

## Multi-Account Management
Switch or verify accounts using:
```bash
heyreach accounts list --check
heyreach accounts test [account_name]
```

## Error Handling
When authentication fails, commands exit with non-zero exit codes and output standardized error envelopes:
- `auth_required` (exit code 3): API key missing, expired, or invalid.
- `no_account` (exit code 7): Specified account does not exist in config.
- `invalid_input` (exit code 6): Bad flag or malformed argument.
- `rate_limited` (exit code 5): HeyReach API rate limit reached (300 req/min).

## Security Best Practices
1. **Never Commit Keys**: Keep `.env` or plaintext token files out of version control.
2. **Use OS Keystore**: The CLI automatically utilizes macOS Keychain, Windows DPAPI, or Linux Secret Service.
3. **Machine Verification**: When writing agent automation scripts, always pass `--json` to reliably capture structured error envelopes.
