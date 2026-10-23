# task-start Specification

## Purpose
How a ClickUp task becomes an OpenSpec change, is reviewed, is implemented, and reports back to
ClickUp. The plugin starts the agent on the task, tracks the task's phase, and drafts every ClickUp
write for approval.

## Requirements

### Requirement: Spec-first start writes a change and no code
A spec-first start SHALL create the task worktree, create the OpenSpec change there, write the whole
ClickUp ticket into the change folder, and prompt the agent to write the proposal, specs, design and
tasks only.

#### Scenario: A spec start
- **WHEN** the user starts a task in spec mode
- **THEN** the worktree holds an OpenSpec change seeded with the ticket
- **AND** the first prompt forbids application code and asks for a validated change

#### Scenario: The whole ticket is in the prompt
- **WHEN** a task has checklists, subtasks, attachments, comments, custom fields or a parent task
- **THEN** the prompt or the seeded ticket file includes them

### Requirement: The board shows the task phase
The board SHALL show each linked task's phase, and SHALL offer the implement action only when a
drafted change waits for implementation.

#### Scenario: A drafted spec
- **WHEN** the spec agent finishes and the link moves to spec drafted
- **THEN** the board shows `spec ✓` for the task
- **AND** the `i` action is offered for that task

#### Scenario: No change yet
- **WHEN** a task has no link or no OpenSpec change
- **THEN** the `i` action is not offered
- **AND** pressing `i` explains that the task needs a spec first

### Requirement: Implement runs the apply workflow
The implement action SHALL send the OpenSpec apply workflow to the agent in the task's existing pane
only when the change passes strict validation and the agent is idle. It SHALL claim the link once,
inside the state lock, and SHALL move it to implementing.

#### Scenario: Implementation starts
- **WHEN** the user presses `i` on a task whose spec is drafted, validates, and whose agent is idle
- **THEN** the agent receives `openspec instructions apply --change <change>`
- **AND** the next finish reports an implementation, not a spec draft

#### Scenario: The change does not validate
- **WHEN** `openspec validate --strict` fails for the change
- **THEN** no prompt is sent
- **AND** the board shows the validation error
- **AND** the link stays spec drafted

#### Scenario: The agent is still working
- **WHEN** the user presses `i` while the spec agent is working
- **THEN** no prompt is sent
- **AND** the board asks the user to wait for the finish

#### Scenario: A second press
- **WHEN** the user presses `i` twice before the board reads the new phase
- **THEN** only the first press claims the link and sends a prompt
- **AND** the second press sends nothing

### Requirement: Reports separate spec drafting from implementation
The hook SHALL draft a spec report for a finished spec phase, and an implementation report for a
finished implementation phase. Neither report is posted without approval.

#### Scenario: Spec finish
- **WHEN** the agent finishes while the link is spec drafting
- **THEN** the draft says the spec is ready for review
- **AND** it uses the spec status when one is configured

#### Scenario: Implementation finish
- **WHEN** the agent finishes while the link is implementing
- **THEN** the draft says the implementation is ready for review
- **AND** it uses the done status when one is configured

### Requirement: A start drafts its status change
A start SHALL NOT write a ClickUp status. When a start status is configured, the start SHALL put a
status-only draft on the task link. The plugin posts the draft only after the user approves it.

#### Scenario: A code start with a status
- **WHEN** a code start succeeds and `CLICKUP_STATUS_STARTED` is set
- **THEN** the task shows a drafted update with that status and no comment
- **AND** ClickUp keeps its current status until the user approves the draft

#### Scenario: A spec start with a status
- **WHEN** a spec start succeeds and `CLICKUP_STATUS_SPEC` is set
- **THEN** the task shows the same kind of status-only draft

#### Scenario: Approval of a status-only draft
- **WHEN** the user approves a draft whose comment is empty
- **THEN** the plugin sets the status
- **AND** it posts no empty comment

#### Scenario: No status configured
- **WHEN** the start status setting is empty or absent
- **THEN** the start drafts nothing

### Requirement: A finish draft summarises the work
A finish draft SHALL say what the agent did. A code finish SHALL list the branch's commits and the
changed-file summary. A spec finish SHALL include the proposal's Why and the task count. A draft is
still posted only after the user approves it.

#### Scenario: A code finish
- **WHEN** a code agent finishes on a branch with commits and changed files
- **THEN** the drafted comment lists the commits and the diff stat
- **AND** the status change stays in the same draft

#### Scenario: A multi-repo finish
- **WHEN** the task has worktrees in several repos
- **THEN** the draft groups the commits and changed files by repo

#### Scenario: A spec finish
- **WHEN** a spec agent finishes
- **THEN** the drafted comment includes the proposal's Why and "N of M" tasks done

#### Scenario: No summaries available
- **WHEN** the worktree or the proposal files cannot be read
- **THEN** the draft keeps the head sentence and posts nothing by itself

### Requirement: A drafted update can be edited before approval
The update dialog SHALL open the draft in `$EDITOR` on `e`, with the comment and a `status:` line.
Saving SHALL return to the dialog with the edited draft and SHALL save it on the link. The draft is
still posted only on Enter.

#### Scenario: Edit the comment and the status
- **WHEN** the user presses `e` on a drafted update and saves the editor with new text
- **THEN** the dialog shows the new comment and status
- **AND** the link stores the edited draft

#### Scenario: Empty the comment
- **WHEN** the edited comment is empty and the status line names a status
- **THEN** approval posts only the status and no comment

#### Scenario: Leave and return
- **WHEN** the user edits a draft and closes the dialog with Esc
- **THEN** pressing `u` again shows the edited draft

#### Scenario: The editor fails or cancels
- **WHEN** the editor exits with a failure or the draft file cannot be read
- **THEN** the original draft stays

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
