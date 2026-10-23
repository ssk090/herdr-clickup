# Proposal: clean up a failed start

## Why

If a start fails after it creates the worktree (OpenSpec setup, agent start, the first prompt), the
worktree, workspace and link stay behind. The next start of the same task then fails because the
branch and path exist.

## What Changes

- A start guard owns the worktrees, workspace, pane and folder. It runs only while the start is in
  flight.
- Before the agent starts, a failure removes the plugin's own setup, even when it left untracked
  files.
- After the agent starts, a failure removes the worktree only when it has no changes. With changes,
  it keeps the worktree and link, and sends a notification that names the close action.
- A success disarms the guard.

## Capabilities

### Modified Capabilities

- `task-start`: a failed start cleans up.

## Impact

`src/app.rs`, `README.md`.
