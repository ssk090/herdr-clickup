## ADDED Requirements

### Requirement: The plugin .env is predictable
For every setting, the plugin SHALL use the `.env` value first and the process environment as a
fallback. An explicit empty status value in `.env` SHALL turn that status off. When `CLICKUP_TOKEN`
is set in both places and differs, the plugin SHALL warn and use the `.env` value.

#### Scenario: Both sources set
- **WHEN** `.env` and the environment set the same key
- **THEN** the `.env` value is used

#### Scenario: A differing token
- **WHEN** both tokens are set and differ
- **THEN** the plugin warns and uses the `.env` token

#### Scenario: The fallback
- **WHEN** `.env` does not set a key
- **THEN** the environment value is used

#### Scenario: A status turned off
- **WHEN** `.env` sets a status to an empty value
- **THEN** that status change is off
