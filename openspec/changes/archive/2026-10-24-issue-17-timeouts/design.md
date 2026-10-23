# Design: timeouts on the agent

## Decisions

- **Set them on the `ureq::Agent`.** The client already builds one agent for rate-limit handling.
  `timeout_connect`, `timeout_recv_response` and `timeout_recv_body` cover the connect, the first
  byte and the rest of the body.
- **10 seconds to connect, 30 seconds to read.** A ClickUp request is small. A slow answer is a
  failure, not something to wait on.
- **Name the request.** The existing context now says `GET /path (10s connect, 30s read)`, and the
  ureq error adds `timeout: ...`. The user sees both.

## Risks

- A very large task page can take more than 30 seconds on a slow link. The page cap and the retry
  keep the request small, and a retry is better than a blocked thread.
- The timeouts apply to every call, including the comment and status writes. A write that times out
  may have reached ClickUp. A retry could duplicate a comment. The user approves each write, and a
  timeout is rare.
