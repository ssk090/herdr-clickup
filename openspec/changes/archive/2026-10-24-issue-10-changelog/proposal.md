# Proposal: a changelog and a release policy

## Why

The version is still 0.1.0 after many feature changes, and there is no changelog. `herdr plugin
update` follows commits, so users cannot see what changed or which version they run. The two version
files can drift.

## What Changes

- Add `CHANGELOG.md` in Keep a Changelog style, with the 0.1.0 features.
- Document the release steps and the version policy in the README.
- Add a test that fails when `Cargo.toml` and `herdr-plugin.toml` name different versions.

No runtime behaviour changes.

## Impact

`CHANGELOG.md` (new), `tests/version.rs` (new), `README.md`.
