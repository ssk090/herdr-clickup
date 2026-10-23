# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-10-24

### Added

- CI on macOS and Linux for fmt, clippy and tests, plus an MSRV job for Rust 1.89.
- A board cache that shows the last data at once and refreshes in the background.
- Rate-limit retries, HTTP timeouts and support for custom task ids.
- A closed-task toggle, a workspace switch, task comments and a status picker.
- Reopening an existing worktree, cleanup after a failed start, stale-link reconcile, a keychain
  token command and predictable `.env` precedence.
- Typed ids, shared JSON helpers and a board state split by responsibility.
- A Linux verification checklist.

### Fixed

- A second `i` press sends no second apply prompt, and `i` waits for an idle agent.
- A moved cursor starts the highlighted repo, not the first.
- A single-repo spec keeps its single worktree and writes no repo list.
- Close checks old links for uncommitted work, tolerates a missing folder and prunes stale records.

## [0.1.0] - 2026-10-24

### Added

- A ClickUp board with sprints, lists, task filters, assignee filters and a task detail view.
- Spec-first starts: an OpenSpec change seeded from the whole ticket, a board phase, `i` to
  implement, and an implementation report separate from the spec report.
- Multi-repo tasks: a task folder with a worktree per repo, `repos.txt`, add a repo, and close.
- Drafted ClickUp updates with commits, changed files, the proposal Why and task count. Nothing is
  posted without approval, and `e` edits a draft in `$EDITOR`.
- A status change is a draft too, and a status-only draft posts no comment.
- Prebuilt macOS and Linux binaries, downloaded on install, with a cargo fallback.
- An MIT license.
