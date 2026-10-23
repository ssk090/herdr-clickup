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
