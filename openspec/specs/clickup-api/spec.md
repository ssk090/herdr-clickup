# clickup-api Specification

## Purpose
How the plugin talks to the ClickUp v2 REST API: requests, retries when ClickUp rate limits a
burst, and the shape of the data it reads.

## Requirements

### Requirement: A rate-limited request waits and retries
When ClickUp answers HTTP 429, the client SHALL wait for `X-RateLimit-Reset`, then `Retry-After`,
then a short backoff, and retry. It SHALL stop after four retries and return a clear error. The
board SHALL send at most four space requests at a time.

#### Scenario: A reset header
- **WHEN** ClickUp answers 429 with `X-RateLimit-Reset`
- **THEN** the client waits until that time, capped at 60 seconds, and retries
- **AND** the burst completes instead of failing

#### Scenario: No reset header
- **WHEN** ClickUp answers 429 without a reset or `Retry-After`
- **THEN** the client backs off for 1, 2, 4 or 8 seconds and retries

#### Scenario: The limit persists
- **WHEN** every attempt is 429
- **THEN** the client stops after four retries and reports the status and body

#### Scenario: A burst of spaces
- **WHEN** the board loads a workspace with many spaces
- **THEN** it sends at most four space requests at a time

### Requirement: A slow request times out
A ClickUp request SHALL time out after 10 seconds to connect and 30 seconds to read a response or
body. A timeout SHALL return an error that names the request and the timeout.

#### Scenario: A connect timeout
- **WHEN** ClickUp does not accept a connection within 10 seconds
- **THEN** the call fails with the request path and a timeout

#### Scenario: A read timeout
- **WHEN** ClickUp accepts the connection but sends no response within 30 seconds
- **THEN** the call fails with the request path and a timeout
