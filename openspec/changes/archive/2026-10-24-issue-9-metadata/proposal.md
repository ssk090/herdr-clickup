# Proposal: package metadata and a pinned MSRV

## Why

`Cargo.toml` has no description, repository or rust-version. The code uses edition 2024 and
let-chains, so an older toolchain fails with confusing parse errors instead of a clear message. The
declared minimum also needs a CI job, or it drifts.

## What Changes

- Add `description`, `repository` and `readme` to `Cargo.toml`.
- Add `rust-version = "1.89"`, the first release with `File::lock` and everything else the code uses.
- Add an `msrv (1.89)` CI job that runs `cargo test --locked` on 1.89.0.
- The README says that building from source needs Rust 1.89 or later.

No runtime behaviour changes.

## Impact

`Cargo.toml`, `.github/workflows/ci.yml`, `README.md`.
