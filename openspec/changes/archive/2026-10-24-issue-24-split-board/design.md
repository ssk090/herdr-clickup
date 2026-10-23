# Design: four modules, one state

## Decisions

- **The state type stays one.** `BoardState` is the single source of the screen. The modules add
  inherent methods and types; there is no second copy of the data.
- **Keys stay in the module that owns them.** Top-level and navigation keys are in `nav`; a dialog
  key is in `modal`; the filter input is a `nav` method. `on_key` only decides which one runs.
- **Messages stay in `load`.** `Msg`, the hierarchy type and `receive` live there. The runtime sends
  messages; `BoardState::receive` delegates.
- **The board keeps the outside.** Drawing, threads and effect execution stay in `board.rs`.
- **Re-exports keep the call sites short.** `board` imports `Source`, `Focus`, `RowKind`, `Modal`
  and `Msg` from `state`.

## Risks

- The modules are still in one crate and can reach each other's public fields. The split is about
  where a change belongs, not a hard boundary.
- A new key belongs in `nav` or `modal`; `on_key` is the one place that must stay small.
