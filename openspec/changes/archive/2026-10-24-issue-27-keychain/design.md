# Design: a command, not a file

## Decisions

- **`sh -c`.** The setting is a shell command, so users can quote arguments and use a pipe. It comes
  from the user's own config, so it has the same trust as the `.env` file.
- **The command wins.** A user who sets the command wants the keychain, not a stale plain token.
  `CLICKUP_TOKEN` stays as the simple path and the fallback.
- **Trim the output.** `security` and `secret-tool` add a newline. An empty result is an error, not
  an empty token.
- **Show the command's stderr.** A locked keychain or a missing entry explains itself in the error.

## Risks

- A command that hangs blocks the plugin start. The user wrote the command.
- The token passes through a shell process. A user who does not trust `sh -c` can keep the file
  token instead.
