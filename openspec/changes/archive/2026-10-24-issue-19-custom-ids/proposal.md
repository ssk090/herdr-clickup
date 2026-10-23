# Proposal: accept ClickUp custom task ids

## Why

Links and `start` assume ClickUp's internal ids. A link with a custom id such as
`app.clickup.com/t/<team>/ABC-123` calls `GET /task/ABC-123` without `custom_task_ids=true` and the
team id, so the lookup fails.

## What Changes

- A task id that contains a dash is treated as a custom id. Every task call adds
  `custom_task_ids=true&team_id=<team>`, including the ticket, comment and status calls.
- An internal id keeps the current paths.
- Links and `start ABC-123` resolve the task, and later calls use the returned internal id.
- Add tests for the path builder and both custom URL forms.

## Capabilities

### Modified Capabilities

- `clickup-api`: a custom task id resolves.

## Impact

`src/clickup.rs`, `tests/clickup.rs`, `tests/link.rs`, `README.md`.
