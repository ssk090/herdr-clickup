# Linux verification

CI runs rustfmt, clippy and the test suite on Ubuntu. The jobs are `test (ubuntu-latest)` and
`msrv (1.89)` in the CI workflow. This file lists the checks that need a real herdr session and a
ClickUp token.

## Before you start

- Linux x86_64 or arm64, herdr 0.9.3 or later, and the plugin installed.
- The `openspec` CLI for a spec start.
- A ClickUp token in `.env` or through `CLICKUP_TOKEN_CMD`.
- `xdg-open` from `xdg-utils` for the browser action.

## Board

- [ ] `prefix+u` opens the board and `q` closes it.
- [ ] Cached data shows first, then refreshes in the background.
- [ ] The side tree expands and a list shows its tasks.
- [ ] `/` filters, `m` shows only mine, `a` picks an assignee.
- [ ] `C` shows closed tasks dimmed.
- [ ] `t` lists the statuses and the chosen status opens a draft.
- [ ] `W` switches workspace and the choice survives a restart.

## Spec start

- [ ] `s` creates the worktree and workspace and starts the agent.
- [ ] The agent writes the OpenSpec change and no code.
- [ ] The task gets a ✉; `u` posts only after Enter.
- [ ] `e` edits the draft; the edit survives Esc and `u`.
- [ ] `i` validates the change, adds the `repos.txt` worktrees and sends apply.

## Code start

- [ ] `c` starts the agent and drafts the status change.
- [ ] The finish draft lists commits or files.
- [ ] A second `i` sends nothing.

## Hooks

- [ ] Closing the pane removes its link.
- [ ] Removing the worktree removes its links.
- [ ] After a herdr restart, `reconcile` drops links to dead panes.

## Multi-repo

- [ ] Two repos create one task folder, one workspace and one agent.
- [ ] `A` adds a repo; `X` closes the task and removes the folder.
