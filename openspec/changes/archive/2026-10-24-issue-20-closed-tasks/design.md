# Design: a second key for a second view

## Decisions

- **A flag on the query.** `include_closed=true` is the only difference. The existing page loop and
  cache work as they are.
- **A separate cache key.** `mine#closed` and `<list>#closed` keep the closed view apart from the
  open view, so toggling back is instant.
- **A status type, not a name.** A task is closed when ClickUp's `status.type` is `closed` or `done`.
  Status names differ per space.
- **Dim the row.** Closed tasks keep their status and phase text but use a dim style, so the table
  stays readable.
- **The tree stays the same.** The toggle applies to the selected source only. A new source shows
  open tasks unless closed is already on.

## Risks

- The closed view can be much larger on an old list. The page cap still holds.
- A custom status with type `custom` is not dimmed even if its name suggests done. The ClickUp type
  is the authority.
