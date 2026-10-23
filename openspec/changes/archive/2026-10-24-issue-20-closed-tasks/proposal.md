# Proposal: show closed tasks

## Why

List queries never set `include_closed`, so a finished task in a sprint cannot be seen. The user
must open ClickUp to check the closed work.

## What Changes

- `my_tasks` and `list_tasks` take an `include_closed` flag and add `include_closed=true`.
- `C` toggles closed tasks for the current source. The list reloads with the flag, and the two views
  cache under separate keys.
- A closed task shows dimmed in the table, and the title says `closed`.
- The README lists the key.

## Capabilities

### New Capabilities

- `task-list`: which tasks a list shows.

## Impact

`src/clickup.rs`, `src/task.rs`, `src/board.rs`, `src/board/state.rs`, `src/main.rs`, `README.md`.
