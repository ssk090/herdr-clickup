# Proposal: run checks on every push and PR

## Why

Nothing runs the tests (101 today), clippy or rustfmt automatically. Review catches style and lint
problems late, and a broken build can reach master.

## What Changes

- A CI workflow runs on macOS (arm64) and Linux (x86_64) for every push to master and every PR.
- Each job runs `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings`
  and `cargo test --locked`.
- The repo is formatted once, so the fmt check starts from a clean tree.
- Master needs both checks before a merge.

No runtime behaviour changes.

## Impact

`.github/workflows/ci.yml` (new), the whole tree for the one-time format, and branch protection.
