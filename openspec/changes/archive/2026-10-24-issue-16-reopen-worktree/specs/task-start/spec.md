## ADDED Requirements

### Requirement: Starting an existing task reopens it
A start SHALL find an existing pane link, worktree or task folder before it creates a new worktree.
It SHALL open what it finds, start the agent again when it exited, and link it.

#### Scenario: A live link
- **WHEN** the task already has a linked pane and agent
- **THEN** the plugin focuses that workspace
- **AND** it creates no second worktree

#### Scenario: An existing worktree without a link
- **WHEN** the branch has a worktree but no link
- **THEN** the plugin opens that worktree in a workspace
- **AND** it starts the agent and links the pane

#### Scenario: An existing task folder
- **WHEN** the task folder and its worktrees exist
- **THEN** the plugin opens a workspace on the folder
- **AND** it links the existing worktrees

#### Scenario: A spec worktree with no change
- **WHEN** a spec start finds a worktree but no OpenSpec change
- **THEN** it stops with a message that names the worktree
