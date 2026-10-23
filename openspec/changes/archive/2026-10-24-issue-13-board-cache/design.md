# Design: cache beside the state file

## Context

`$HERDR_PLUGIN_STATE_DIR/panes.json` already holds the pane links. The board's tasks and hierarchy are
serde-friendly Rust types. A board open must still refresh, because the cache can be old.

## Decisions

- **One file per source.** `cache/hierarchy.json` holds the spaces, lists and user id.
  `cache/tasks-<key>.json` holds one source's tasks. The key is sanitized for a file name.
- **Best effort writes.** A read-only or full state dir must not break the board. A write to a temp
  file and a rename keeps a reader from seeing half a file.
- **A corrupt cache is no cache.** A parse error returns `None`, so the board loads normally.
- **Stale marks per source.** `BoardState::stale` holds `hierarchy` and source keys. A successful
  message clears its key. The panels say `(cached)` while a stale key is present and no load is
  running.
- **The cache saves on every successful load.** The next open starts from the newest data.

## Risks

- A cache from a different workspace can show for a moment. The background refresh replaces it.
- Task lists can be large. The files are one JSON array each, and the board only reads one source at
  open.
