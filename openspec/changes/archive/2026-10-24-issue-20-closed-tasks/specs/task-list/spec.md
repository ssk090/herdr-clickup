## ADDED Requirements

### Requirement: Closed tasks can be shown
The board SHALL show only open tasks by default. `C` SHALL toggle closed tasks for the current
source, request them with `include_closed=true`, and show them dimmed. A second `C` SHALL return to
open tasks only.

#### Scenario: Show closed tasks
- **WHEN** the user presses `C` on a list
- **THEN** the list reloads with closed tasks
- **AND** the title says `closed`
- **AND** the closed tasks show dimmed

#### Scenario: Hide them again
- **WHEN** the user presses `C` again
- **THEN** the list shows open tasks only

#### Scenario: The two views cache apart
- **WHEN** the user toggles back and forth
- **THEN** each view uses its own cache key
