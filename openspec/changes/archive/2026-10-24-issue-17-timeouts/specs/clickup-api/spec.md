## ADDED Requirements

### Requirement: A slow request times out
A ClickUp request SHALL time out after 10 seconds to connect and 30 seconds to read a response or
body. A timeout SHALL return an error that names the request and the timeout.

#### Scenario: A connect timeout
- **WHEN** ClickUp does not accept a connection within 10 seconds
- **THEN** the call fails with the request path and a timeout

#### Scenario: A read timeout
- **WHEN** ClickUp accepts the connection but sends no response within 30 seconds
- **THEN** the call fails with the request path and a timeout
