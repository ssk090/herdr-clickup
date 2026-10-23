# Design: status as data, retry on 429

## Decisions

- **`http_status_as_error(false)`.** ureq returns a response for 4xx and 5xx, so the client can read
  the status and the rate-limit headers before it builds an error.
- **One agent per client.** The client builds a `ureq::Agent` once, so connections are reused across
  the pages and retries.
- **Wait order.** `X-RateLimit-Reset` is an epoch in seconds. `Retry-After` is a number of seconds.
  When both are absent or in the past, the wait is 1, 2, 4, 8 or 16 seconds by attempt, capped at 60.
- **Retry writes too.** ClickUp does not process a request it rate limits, so a retried comment or
  status change is safe.
- **Four spaces at a time.** The hierarchy load chunks the spaces. A big workspace no longer sends
  every request at once.

## Risks

- A reset header in milliseconds is read as seconds and capped at 60. The next attempt may still be
  rate limited, and the fourth ends in an error.
- A long wait blocks that request thread. The board and a start already run requests on their own
  threads.
