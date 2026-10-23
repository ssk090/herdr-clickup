## ADDED Requirements

### Requirement: The detail view shows comments
When the user opens a task, the board SHALL fetch the ticket and show its comments with author and
date. A failed fetch SHALL report the error and keep the description.

#### Scenario: A task with comments
- **WHEN** the detail view opens for a task with comments
- **THEN** the comments show with author and date

#### Scenario: No comments
- **WHEN** the ticket has no comments
- **THEN** the view says `No comments.`

#### Scenario: The fetch fails
- **WHEN** the ticket cannot be loaded
- **THEN** the board reports the error
- **AND** the description still shows

### Requirement: A status change waits for approval
The board SHALL list the task's list statuses on `t`. Choosing a status SHALL open a status-only
draft, and the status SHALL change only when the user approves that draft.

#### Scenario: Choose a status
- **WHEN** the user presses `t` and picks a status
- **THEN** a status-only draft opens
- **AND** ClickUp keeps its status until the user approves the draft

#### Scenario: A task without a list
- **WHEN** the task has no list id or the list has no statuses
- **THEN** the board reports that there are no statuses
