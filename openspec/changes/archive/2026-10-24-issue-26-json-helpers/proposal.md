# Proposal: one json helper module

## Why

`text`, `arr` and `num` helpers, and `get(..).and_then(Value::as_str).map(str::to_string)` chains,
are repeated across `task`, `ticket`, `hierarchy`, `clickup`, `herdr` and `hook`. A fix in one copy
does not reach the others.

## What Changes

- Add `src/json.rs` with `text`, `arr`, `num`, `flag`, `at` and `u64_at`.
- Use it in the ClickUp parsers, the herdr call parsing and the hook's event reading.
- Remove the local copies.
- Add tests for the helpers.

## Capabilities

None: this is a refactor. The change sets `skip_specs`.

## Impact

`src/json.rs` (new), `src/lib.rs`, `src/task.rs`, `src/ticket.rs`, `src/hierarchy.rs`,
`src/clickup.rs`, `src/herdr.rs`, `src/hook.rs`, `tests/json.rs`.
