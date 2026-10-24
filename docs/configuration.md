# Configuration

The plugin has no fixed ClickUp workspace, space, status, agent or repo. You choose each one in the
plugin `.env`, on the board, or in a start dialog.

## Where the file lives

```sh
herdr plugin config-dir ssk090.clickup
# create .env in that directory
```

## Full example

```sh
# Required. A personal token from ClickUp Settings -> Apps.
CLICKUP_TOKEN=pk_...

# Optional. A command that prints the token, for a keychain. It wins over CLICKUP_TOKEN.
# CLICKUP_TOKEN_CMD=security find-generic-password -s herdr-clickup -w

# Optional. The workspace (team) id. Without it, the plugin uses the first workspace.
# The board's W key remembers a choice that wins over this value.
# CLICKUP_TEAM_ID=9012345678

# Optional. Comma-separated space ids. Without it, the board shows every space.
# The id is the number after /s/ in a space URL.
# CLICKUP_SPACE_IDS=11111111,22222222

# Optional. A herdr agent kind. Without it, the first installed agent leads.
# CLICKUP_AGENT=codex

# Optional. Status drafts. Names differ per space. An empty value turns the draft off.
# CLICKUP_STATUS_STARTED=in progress
# CLICKUP_STATUS_DONE=review
# CLICKUP_STATUS_SPEC=spec review
```

## Where to find the ids

- **Team id**: the number in a ClickUp URL before `/v/` or `/s/`, for example
  `app.clickup.com/9012345678/...`. The board's `W` picker shows the workspace names.
- **Space id**: the number after `/s/` in a space URL.
- **Status names**: Space settings -> Statuses.
- **Task ids**: the internal id in a task URL, or the custom id such as `ABC-123`.

## What you choose at runtime

- **Workspace**: `W` on the board.
- **Agent and repos**: the start flow. Repos come from your open herdr workspaces.
- **Repos for a spec**: `repos.txt` in the change folder. It holds absolute repo paths, one per line.
- **Custom task ids**: a link or `start ABC-123` works. The plugin sends the custom id flags.
- **Status names**: `t` in the task detail. The plugin reads the list's statuses.
- **Token source**: `.env` wins over the process environment. A difference prints a warning.

## Paths

- Task folders for 2+ repos: `~/.herdr/tasks/<branch>/`.
- State and cache: `$HERDR_PLUGIN_STATE_DIR/`.
- The plugin does not write to your repos outside a task worktree.

## No company data

The runtime has no fixed workspace, space, status or repo. The tests use demo names such as
`Demo Co`, `webui` and `api-hub`.
