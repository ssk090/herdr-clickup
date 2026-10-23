# board-cache Specification

## Purpose
What the board shows before ClickUp answers: the last hierarchy and task lists, marked cached, then
replaced by a background refresh.

## Requirements

### Requirement: The board opens from cache
The board SHALL show the last hierarchy and the default source's tasks before ClickUp answers, and
SHALL mark them as cached. It SHALL refresh them in the background.

#### Scenario: A warm cache
- **WHEN** the board opens and a cache exists
- **THEN** the cached spaces, lists and my tasks show at once
- **AND** the panels say `(cached)` until fresh data arrives
- **AND** the board refreshes in the background

#### Scenario: Fresh data
- **WHEN** the hierarchy or a task source loads
- **THEN** the result replaces the cache and the cached mark clears
- **AND** the result is saved for the next open

#### Scenario: A cold or corrupt cache
- **WHEN** there is no cache or it cannot be read
- **THEN** the board loads normally with no cached mark
