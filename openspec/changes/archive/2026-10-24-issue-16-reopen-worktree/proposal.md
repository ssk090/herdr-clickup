# Proposal: reopen an existing task instead of failing

## Why

Starting a task whose `CU-<id>-...` branch and worktree already exist fails with an "already exists"
error. The worktree may hold work in progress, and the user must open it in Herdr by hand.

## What Changes

- A start first looks for the task: an existing pane link, an existing worktree of the branch, or an
  existing task folder.
- With a live link it focuses the workspace, and starts the agent again if it exited.
- Without a link it opens the existing worktree, or a workspace on the task folder, starts the agent
  and links it.
- A spec reopen uses the existing OpenSpec change. A worktree with no change gives a clear error
  instead of a branch error.
- Only a start with nothing to reopen creates a new worktree.

## Capabilities

### Modified Capabilities

- `task-start`: starting an existing task reopens it.

## Impact

`src/app.rs`, `src/herdr.rs`, `src/taskspace.rs`, `README.md`.
