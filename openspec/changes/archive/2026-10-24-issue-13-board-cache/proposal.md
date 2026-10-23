# Proposal: open the board from cache

## Why

Every board open downloads the spaces, lists and tasks. Nothing shows for a few seconds, and a slow
or rate-limited ClickUp makes the wait longer. The last data is still on disk from the previous run.

## What Changes

- The board caches the hierarchy and each source's tasks in `$HERDR_PLUGIN_STATE_DIR/cache/`.
- On open it shows the cache at once, marks the side and task panels `(cached)`, and refreshes in the
  background.
- A successful load replaces the cache and clears the mark.
- A missing or corrupt cache falls back to a normal load. Cache writes are best effort.

## Capabilities

### New Capabilities

- `board-cache`: what the board shows before ClickUp answers.

## Impact

`src/cache.rs` (new), `src/board.rs`, `src/board/state.rs`, `src/hierarchy.rs`, `src/task.rs`,
`README.md`.
