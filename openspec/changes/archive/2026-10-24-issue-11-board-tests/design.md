# Design: state in, effects out

## Context

The board separates background work from drawing with a channel. The key handler and the state still
live with the threads and the terminal, so tests need a terminal.

## Decisions

- **One state struct for the screen.** `BoardState` owns the sources, tasks, selection, filter,
  modals, links and drafts. It has no ClickUp client, no channel and no terminal.
- **Keys return effects.** `on_key` mutates the state and returns `Vec<Effect>`. A pure state change
  (move, expand, filter) returns nothing. `Start`, `Implement`, `PostUpdate`, `CloseTask`, `OpenUrl`
  and `EditDraft` leave the module.
- **The runtime executes effects.** `src/board.rs` maps an effect to a thread, a file read or a
  herdr call. It puts the results back through `Msg`, which `BoardState::receive` applies.
- **Choices come from the runtime.** The state cannot read the file system, so it asks for an agent
  or repo choice with an effect. The runtime gathers the choices and calls `open_agent` or
  `open_repo`.
- **Tests build the state directly.** They insert tasks and links, press keys and assert effects or
  visible tasks. No terminal, no network, no threads.

## Risks

- The state module is still large. #24 splits loading, navigation and modals into separate modules.
- An effect that is not handled leaves the screen waiting. The `Effect` enum is exhaustive in the
  runtime's `match`, so a new effect is a compile error there.
