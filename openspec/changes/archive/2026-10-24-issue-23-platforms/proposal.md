# Proposal: document the supported platforms

## Why

The manifest excludes Windows, and the code uses Unix permissions and `sh` wrappers, but the README
does not say so. A Windows user could install it and hit confusing failures.

## What Changes

- Add a Platforms section: macOS and Linux only, Windows not supported.
- No code changes.

## Impact

`README.md`.
