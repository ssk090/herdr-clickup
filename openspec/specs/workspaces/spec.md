# workspaces Specification

## Purpose
Which ClickUp workspace the plugin reads, how the board switches it, and how the choice is
remembered.

## Requirements

### Requirement: The board can switch workspace
The board SHALL list the workspaces the token can see on `W`. Choosing one SHALL remember it, clear
the old cache, and reload the board. A remembered choice SHALL win over `CLICKUP_TEAM_ID`.

#### Scenario: Switch workspace
- **WHEN** the user presses `W` and picks a workspace
- **THEN** the board reloads the hierarchy and my tasks from that workspace
- **AND** the choice is saved for later runs

#### Scenario: No remembered choice
- **WHEN** no workspace was chosen on the board
- **THEN** `CLICKUP_TEAM_ID` is the default
- **AND** without it, the first workspace is used

#### Scenario: The old cache
- **WHEN** the workspace changes
- **THEN** the old hierarchy and task cache are removed
