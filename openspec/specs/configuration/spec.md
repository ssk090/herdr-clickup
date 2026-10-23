# configuration Specification

## Purpose
How the plugin reads its settings and token, including a keychain command as the token source.

## Requirements

### Requirement: The token can come from a command
When `CLICKUP_TOKEN_CMD` is set, the plugin SHALL run it with `sh -c` and use its trimmed stdout as
the token. A failure or empty output SHALL stop the plugin with that error. The command SHALL win
over `CLICKUP_TOKEN`.

#### Scenario: A keychain command
- **WHEN** `CLICKUP_TOKEN_CMD` prints a token
- **THEN** the plugin uses that token

#### Scenario: A failing command
- **WHEN** the command exits non-zero or prints nothing
- **THEN** the plugin stops with the command's error

#### Scenario: Both sources set
- **WHEN** `CLICKUP_TOKEN_CMD` and `CLICKUP_TOKEN` are both set
- **THEN** the command wins

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
