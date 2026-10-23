# Proposal: typed ids

## Why

Task, pane, workspace and list ids are all `String` or `&str`. A positional call can swap them and
still compile, and `Source::key` mixes the magic `"mine"` with list ids.

## What Changes

- Add `TaskId`, `PaneId`, `WorkspaceId` and `ListId` in `src/ids.rs`. They are transparent in JSON,
  compare to `str`, and work as map keys.
- Use `TaskId` in `Task.id` and `Link.task_id`, `ListId` in `ListRef.id`, and `PaneId` in the pane
  map, the herdr worktree and the hook decisions.
- Replace `Source::key` with the typed `SourceKey`, which also carries the closed flag.
- The cache keys keep their old text, so the existing cache still loads.

## Capabilities

None: this is a refactor. The change sets `skip_specs`.

## Impact

`src/ids.rs` (new), `src/task.rs`, `src/hierarchy.rs`, `src/hook.rs`, `src/herdr.rs`,
`src/store.rs`, `src/app.rs`, `src/board.rs`, `src/board/state.rs`, tests.
