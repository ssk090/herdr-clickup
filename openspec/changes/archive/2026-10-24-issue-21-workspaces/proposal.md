# Proposal: switch ClickUp workspace

## Why

Without `CLICKUP_TEAM_ID` the plugin uses the first workspace the token can see. A user in more than
one workspace has no way to switch, and a wrong default works on the wrong tasks.

## What Changes

- `W` lists the token's workspaces. Choosing one remembers it, clears the cache, and reloads the
  board from that workspace.
- The remember file wins over `CLICKUP_TEAM_ID`; the environment value is the default when no
  choice was made.
- CLI starts use the remembered workspace too.
- The setting table and key list say so.

## Capabilities

### New Capabilities

- `workspaces`: which ClickUp workspace the plugin reads.

## Impact

`src/clickup.rs`, `src/cache.rs`, `src/app.rs`, `src/board.rs`, `src/board/state.rs`, `README.md`.
