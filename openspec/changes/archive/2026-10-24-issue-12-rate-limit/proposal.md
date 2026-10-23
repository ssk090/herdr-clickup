# Proposal: wait out ClickUp rate limits

## Why

ClickUp allows about 100 requests a minute per token. Loading the hierarchy (two requests per space,
in parallel), paging tasks and fetching tickets can hit that limit. A 429 currently surfaces as a
plain error and the board or a start fails.

## What Changes

- The ClickUp client sends requests with HTTP status as data, not an error.
- On a 429 it waits for `X-RateLimit-Reset`, then `Retry-After`, then a short exponential backoff,
  and retries up to four times.
- A 429 waits at most 60 seconds per attempt.
- The board loads at most four spaces at a time, so a large workspace does not burst the limit.

## Capabilities

### New Capabilities

- `clickup-api`: how the plugin talks to the ClickUp REST API.

## Impact

`src/clickup.rs`, `src/board.rs`, `README.md`.
