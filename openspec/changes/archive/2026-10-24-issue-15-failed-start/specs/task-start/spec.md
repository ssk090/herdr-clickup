## ADDED Requirements

### Requirement: A failed start cleans up
A start that fails after it creates a worktree SHALL remove the worktrees and workspace when they
hold no changes. Before the agent starts, the plugin's own setup SHALL not count as a change. When
changes exist, the start SHALL keep the worktrees and the link, and SHALL tell the user how to close
them.

#### Scenario: A failure before the agent starts
- **WHEN** the OpenSpec setup or the agent start fails before any agent work
- **THEN** the worktrees and workspace are removed
- **AND** the pane link is forgotten

#### Scenario: A failure after the agent wrote changes
- **WHEN** the agent wrote files before a later step failed
- **THEN** the worktrees and link are kept
- **AND** a notification names the worktree and the `X` close action

#### Scenario: A clean worktree after an agent failure
- **WHEN** the agent started but wrote no changes and a later step fails
- **THEN** the worktrees and workspace are removed
