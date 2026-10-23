# Design: find before create

## Decisions

- **The link is the first check.** A running task gets focus. If `agent get` fails, the agent
  exited, so the plugin starts it again and sends a continuation prompt.
- **Prune before the link check.** A link to a dead pane must not hide the worktree. The plugin
  runs the #14 prune first.
- **`git worktree list` finds the branch.** For one repo, `find_worktree` reads the porcelain list
  and returns the checkout whose `branch` is `refs/heads/CU-...`.
- **`worktrees_in` finds a task folder's checkouts.** Each subdirectory that is a git worktree gives
  its main repo path and its checkout path.
- **A spec reopen needs the change.** The change folder must exist. Otherwise the start stops with
  a message that names the worktree and the two ways out: open it, or close the task.
- **No guard on reopen.** The worktree existed before this start, so a failure must not remove it.
  The fresh path keeps its #15 guard.

## Risks

- `worktree open` on a checkout that is already open may focus it or open a second workspace,
  depending on Herdr. A duplicate workspace is harmless.
- The continuation prompt does not name the host repo for a multi-repo spec. The agent finds
  `openspec/` in the only worktree or in `repos.txt`.
