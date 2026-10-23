# Design: the plugin owns its settings

## Decisions

- **File first, environment second.** The `.env` belongs to the plugin. The environment belongs to
  the shell or the Herdr server and may carry a value the user forgot.
- **One `pick` helper.** It treats an empty value as unset. The status settings use their own match,
  because an empty status is a deliberate "off" and must not fall through.
- **Warn on a token mismatch.** The warning goes to stderr, which Herdr records in the plugin log.
  The file value still wins.
- **The command stays first.** `CLICKUP_TOKEN_CMD` is checked before the static token, wherever each
  is set.

## Risks

- A user who set an environment value to override a stale `.env` now needs to edit the `.env`. The
  warning names the problem for the token; the README states the rule for every setting.
- An empty `.env` value for a non-status setting falls through to the environment. That keeps the
  old "unset" meaning.
