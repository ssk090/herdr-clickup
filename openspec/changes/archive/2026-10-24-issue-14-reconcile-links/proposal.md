# Proposal: drop stale pane links

## Why

A link in `panes.json` is removed only on `pane.closed` or `worktree.removed`. If Herdr restarts, a
pane dies without an event, or an event is missed (Herdr does not replay them), the link stays
forever and can later match a reused situation.

## What Changes

- A `reconcile` command lists the live panes and removes every link whose pane is not in that list.
- Herdr runs it from a `[[startup]]` hook after it restores the session.
- The board also runs it on open.
- A failed pane list leaves the links alone.

## Capabilities

### New Capabilities

- `pane-links`: the life of a pane-to-task link.

## Impact

`herdr-plugin.toml`, `src/main.rs`, `src/board.rs`, `src/herdr.rs`, `src/store.rs`, `README.md`.
