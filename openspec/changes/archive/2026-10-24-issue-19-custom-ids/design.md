# Design: a dash means custom

## Decisions

- **The dash is the signal.** ClickUp's internal ids have no dash. Custom ids use a prefix and a
  number, such as `ABC-123`. The client does not spend a failed request to find out.
- **One path builder.** `task_path` adds the custom query, the suffix (`/comment`) and any extra
  query in the right order. Task, ticket, status and comment calls use it.
- **The team is only read for a custom id.** An internal id does not pay for `GET /team`.
- **The resolved task carries the internal id.** After `task` or `ticket`, the rest of the plugin
  uses `Task.id`, so a custom id is resolved once.
- **The link parser already keeps the last path segment.** A team-scoped custom URL gives `ABC-123`.
  The new tests pin that down.

## Risks

- A custom id with no dash is read as an internal id. ClickUp's custom ids normally contain a dash.
- A dash in a future internal id would trigger the custom query. The API would reject it, and the
  error names the path.
