# Proposal: make every setting visible and generic

## Why

The plugin has no fixed company data in the runtime, but the tests used the author's company and
people. A new user also has to infer some settings from the code. The README table covers the main
values, but not where to find the ids or what the board asks at runtime.

## What Changes

- Replace the sample company and people in the tests with demo names.
- Add `docs/configuration.md` with a full `.env` example, the id locations and the runtime choices.
- Link it from the README and state that the plugin has no fixed workspace, space, status, agent or
  repo.
- No runtime changes.

## Impact

`docs/configuration.md` (new), `README.md`, test fixtures.
