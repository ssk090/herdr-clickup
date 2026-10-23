# Proposal: a tested board state machine

## Why

`src/board.rs` holds loading, navigation, filtering, five modals and the drawing, and it had no
tests. Bugs in modal flow, selection and filtering reached review because only a terminal could
exercise them. The file is also over 1400 lines, so every change touches the same struct.

## What Changes

- A new `board::state` module holds `BoardState`, the modals, the selection, the filter and the
  links. `BoardState::on_key` returns a list of `Effect`s for work outside the state. The effects
  load a source, start a task, implement, post, close, open a URL or run the editor.
- `src/board.rs` keeps the drawing, the threads and the execution of effects. It holds a
  `BoardState` instead of every field.
- Messages update the state through `BoardState::receive`.
- Tests cover filtering, selection bounds, side expansion and source loading. They also cover the
  agent and repo modals, implement, update post/discard/edit, detail actions, the assignee filter
  and messages.
- The update dialog now names the `e` edit key, which the #5 change missed.

No user-facing behaviour changes.

## Capabilities

None: this is a refactor. The change sets `skip_specs`.

## Impact

`src/board.rs`, `src/board/state.rs` (new).
