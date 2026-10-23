# Proposal: Linux verification

## Why

The manifest declares Linux, and CI runs the tests on Ubuntu, but the board, a start and the hooks
need a real herdr session and a ClickUp token. There is no written checklist for that run.

## What Changes

- Add `docs/linux.md` with the manual checks for the board, spec start, code start, hooks and a
  multi-repo task.
- The README links to it and names the Linux CI jobs.
- No code changes.

## Impact

`docs/linux.md` (new), `README.md`.
