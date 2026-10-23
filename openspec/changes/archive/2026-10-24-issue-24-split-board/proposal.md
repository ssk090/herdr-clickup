# Proposal: split the board state by responsibility

## Why

`src/board.rs` and its state held loading, navigation, filtering, five modals and the drawing in one
place. Any change touched the same struct, and the state file was over 1600 lines.

## What Changes

- `board/state.rs` keeps `BoardState`, `Effect`, the open-modal methods, the state links and the
  key entry point.
- `board/state/nav.rs` holds the source keys, the side tree, the task table, filtering and the keys
  that move them.
- `board/state/modal.rs` holds the `Modal` enum and the keys each dialog takes.
- `board/state/load.rs` holds the messages from the ClickUp threads and the loading flags they
  update.
- `board.rs` keeps the drawing, the threads and the effect execution.

No behaviour changes.

## Capabilities

None: this is a refactor. The change sets `skip_specs`.

## Impact

`src/board.rs`, `src/board/state.rs`, `src/board/state/nav.rs` (new), `src/board/state/modal.rs`
(new), `src/board/state/load.rs` (new).
