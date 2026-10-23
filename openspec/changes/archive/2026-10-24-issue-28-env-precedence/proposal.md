# Proposal: make the plugin .env predictable

## Why

`Config::load` prefers process environment variables over the `.env` file. A `CLICKUP_TOKEN`
exported in a shell profile, or inherited by the Herdr server, silently wins over the plugin's own
config, possibly a different account.

## What Changes

- Every setting reads the plugin `.env` first. The process environment is the fallback for values
  that `.env` does not set.
- An explicit empty status value in `.env` still turns that status off and does not fall through.
- When `CLICKUP_TOKEN` is set in both places and differs, the plugin warns and uses the `.env` value.
- The README says which source wins.

## Capabilities

### Modified Capabilities

- `configuration`: the plugin .env is predictable.

## Impact

`src/config.rs`, `README.md`, `tests/config.rs`.
