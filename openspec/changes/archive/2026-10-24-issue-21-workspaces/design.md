# Design: remember the choice, clear the cache

## Decisions

- **The board lists teams.** `GET /team` returns every workspace with an id and a name. `W` loads
  them on a thread and opens a picker. The state never holds the token.
- **The choice is a state file.** `store::remember("team", id)` writes it. `app::client` reads it
  first, then `CLICKUP_TEAM_ID`, then the API's first workspace.
- **The cache cannot cross workspaces.** A switch removes `cache/` and reloads the hierarchy and my
  tasks. The pane links stay, because task ids are unique across workspaces.
- **The client is rebuilt.** `ClickUp::new` takes the new team id. Custom task id lookups then use
  the right team.
- **A bad remembered team is recoverable.** `W` does not need the current team, so the user can
  switch away from a workspace that no longer exists.

## Risks

- A switch drops the cached data, so the first load after a switch is not instant.
- The picker shows names only. Two workspaces with the same name are hard to tell apart.
