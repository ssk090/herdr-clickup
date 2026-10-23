# Proposal: time out a slow ClickUp call

## Why

The ClickUp client has no timeout. A hung request blocks its thread forever, so a board load or a
start never finishes and the user gets no error.

## What Changes

- Every ClickUp request uses a 10-second connect timeout and a 30-second read timeout.
- A timeout returns an error that names the request and the timeout.
- The README states the limits.

## Capabilities

### Modified Capabilities

- `clickup-api`: a slow request times out.

## Impact

`src/clickup.rs`, `README.md`.
