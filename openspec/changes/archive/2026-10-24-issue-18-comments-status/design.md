# Design: load the ticket, draft the status

## Decisions

- **Load on open.** The detail view emits `LoadDetail` when it opens. The runtime fetches the ticket
  and the list statuses on a thread, then sends one `Msg::Detail`. The UI never blocks.
- **Comments are read from the ticket.** The ticket fetch already carries them. The view shows the
  author, the date and each line of text.
- **Statuses come from the list.** `GET /list/{id}` returns the statuses in order. The task keeps
  its `list_id` from the task response.
- **The picker makes a draft.** Choosing a status opens the existing update dialog with an empty
  comment and that status. Enter posts it through `post_update`, which skips the empty comment. This
  keeps one approval path for every ClickUp write.
- **A failed fetch keeps the description.** The error goes to the status line; the rest of the view
  stays.

## Risks

- A ticket with many long comments can overflow the modal. The paragraph clips at the bottom.
- A task from a cache written before `list_id` existed has none, so `t` reports no statuses. The
  next refresh stores the new field.
