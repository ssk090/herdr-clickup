# Design: one way to read a field

## Decisions

- **Keep the shapes.** A missing string is an empty string, a missing array is an empty slice and a
  missing number is `None`, as before. The call sites do not change behaviour.
- **`num` accepts both forms.** ClickUp sends some numbers as strings and others as numbers.
- **`at` and `u64_at` cover pointers.** The nested fields (`/status/status`, `/user/id`) no longer
  repeat the pointer chain.
- **`flag` covers a boolean.** The task page's `last_page` is the one caller.
- **`arr` returns a slice.** No allocation; the callers only iterate.

## Risks

- A future field with a different missing-value rule needs its own helper rather than a special
  case in these.
