# Proposal: read the token from a keychain

## Why

The token lives in plain text in `$HERDR_PLUGIN_CONFIG_DIR/.env` (mode 600). That is acceptable but
not ideal. A keychain keeps it out of a file.

## What Changes

- Read `CLICKUP_TOKEN_CMD` and run it with `sh -c`. Its trimmed stdout is the token.
- A failing command or empty output stops the plugin with a clear error.
- The command wins over `CLICKUP_TOKEN`.
- The README shows the macOS keychain and Linux `secret-tool` commands.

## Capabilities

### New Capabilities

- `configuration`: how the plugin reads its settings and token.

## Impact

`src/config.rs`, `README.md`, `tests/config.rs`.
