## ADDED Requirements

### Requirement: A custom task id resolves
When a task id contains a dash, the client SHALL send `custom_task_ids=true` and the team id on
every call for that task. A ClickUp link and a `start` command SHALL accept a custom id.

#### Scenario: A custom link
- **WHEN** the user opens `https://app.clickup.com/t/<team>/ABC-123`
- **THEN** the plugin resolves `ABC-123` with the custom id flags

#### Scenario: A custom start
- **WHEN** the user runs `start ABC-123`
- **THEN** the task loads
- **AND** later calls use the internal id from the response

#### Scenario: An internal id
- **WHEN** the id has no dash
- **THEN** the client sends no custom id flags
