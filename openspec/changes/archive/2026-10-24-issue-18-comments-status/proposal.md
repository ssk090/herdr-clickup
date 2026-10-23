# Proposal: comments and status in the detail view

## Why

The task detail shows only the description. There is no way to read the comments or change the
status from the board, so the user must switch to ClickUp for both.

## What Changes

- Opening a task loads its ticket and shows the comments with author and date.
- `t` lists the task's list statuses. Choosing one opens a status-only draft in the existing update
  dialog, and Enter posts it. Nothing is written before that approval.
- The task keeps its list id, so the board knows where to read the statuses.

## Capabilities

### New Capabilities

- `task-detail`: what the board shows for one task, and how it changes a status.

## Impact

`src/board.rs`, `src/board/state.rs`, `src/clickup.rs`, `src/task.rs`, `src/ticket.rs`, `README.md`.
