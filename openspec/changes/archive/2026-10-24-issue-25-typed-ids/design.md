# Design: a small macro, transparent serde

## Decisions

- **One macro, four types.** The four ids share the same operations, so a macro keeps them in step.
- **Transparent JSON.** Old `panes.json`, `hierarchy.json` and task caches still load. A map keyed by
  `PaneId` reads and writes a plain string key.
- **`Borrow<str>`.** A map or set accepts a plain key for lookup, which keeps the call sites short.
- **`PartialEq<&str>`.** Tests and comparisons read naturally without `as_str()` everywhere.
- **`SourceKey` replaces the magic string.** It carries `Mine`/`List` and the closed flag. The cache
  key method keeps the old file names.

## Risks

- A new id type needs an `id_type!` line and a place in the affected structs.
- `Borrow<str>` means a lookup with a plain string can miss a typed key only if the text differs.
