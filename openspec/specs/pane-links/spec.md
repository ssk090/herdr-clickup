# pane-links Specification

## Purpose
The life of a pane-to-task link: when it is written, what it carries, and how links to panes that
no longer exist are removed.

## Requirements

### Requirement: Stale pane links are removed
On server startup and when the board opens, the plugin SHALL list the live panes and remove every
link whose pane is not in that list. A failed pane list SHALL leave the links alone.

#### Scenario: A pane is gone
- **WHEN** Herdr restarts and a linked pane no longer exists
- **THEN** its link is removed

#### Scenario: A live pane
- **WHEN** a linked pane still exists
- **THEN** its link stays

#### Scenario: The pane list fails
- **WHEN** `herdr pane list` fails
- **THEN** the links stay
