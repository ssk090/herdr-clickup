# Design: a guard with a drop

## Context

`app::start` creates a worktree and workspace, then runs several fallible steps. Each `?` used to
return and leave the files behind.

## Decisions

- **A `StartGuard` with `Drop`.** The guard holds the workspace, pane, folder and worktrees. Every
  early return drops it. A success calls `disarm`.
- **A flag for the agent.** Before the link and the agent start, a failure removes the worktrees even
  when OpenSpec setup left untracked files. After the agent starts, a dirty worktree is real work.
- **Clean after the agent means remove.** An agent that wrote nothing leaves a worktree equal to its
  base, so the guard removes it.
- **Keep means keep the link.** The board can close the kept worktree with `X`. The notification
  names the path and the action.
- **Removal is best effort.** A failed `git worktree remove` keeps the worktree and sends the keep
  notification. The link stays so the user can retry close.

## Risks

- A pre-agent failure during OpenSpec setup deletes the half-written change. That is the point: the
  user can start again on a clean branch.
- The notification and the error both show. The error says why the start failed, and the
  notification says what happened to the files.
