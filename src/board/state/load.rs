//! Board loading: the messages that arrive from the ClickUp threads.
use super::{BoardState, Modal, REFRESHING, SourceKey};
use crate::{hierarchy::ListRef, ids::TaskId, task::Task, ticket::Ticket};
use anyhow::Result;

/// Spaces as `(id, name)`, all their lists, the token user's id, and how many spaces failed to load.
pub type Hierarchy = (Vec<(String, String)>, Vec<ListRef>, Option<u64>, usize);

pub enum Msg {
    Hierarchy(Result<Hierarchy>),
    Tasks(SourceKey, Result<Vec<Task>>),
    Started(TaskId, Result<String>),
    /// Something the user must do for a start that is still running.
    Progress(String),
    /// An approved update was posted (or failed).
    Posted(Result<String>),
    /// The apply workflow was sent to an agent (or could not be).
    Implemented(TaskId, Result<String>),
    /// A repo was added to a running task (or could not be).
    Added(Result<String>),
    /// A task was closed and its worktrees removed (or could not be).
    Closed(Result<String>),
    /// The ticket and the list's statuses for the detail view.
    Detail(TaskId, Result<Box<Ticket>>, Result<Vec<String>>),
    /// The workspaces this token can see.
    Teams(Result<Vec<(String, String)>>),
}

impl BoardState {
    pub(super) fn done_refreshing(&mut self) {
        if self.status == REFRESHING && !self.loading_hierarchy && self.loading_tasks.is_empty() {
            self.status.clear();
        }
    }

    // ---- derived views ----
}

pub(super) fn receive(state: &mut BoardState, msg: Msg) -> bool {
    match msg {
        Msg::Hierarchy(Ok((spaces, lists, me, failed))) => {
            state.loading_hierarchy = false;
            state.done_refreshing();
            if failed > 0 {
                state.status = format!("{failed} space(s) could not be loaded; press r to retry");
            }
            state.spaces = spaces;
            state.lists = lists;
            state.me = me;
            state.expand_single_space();
            state.stale_hierarchy = false;
            false
        }
        Msg::Hierarchy(Err(e)) => {
            state.loading_hierarchy = false;
            state.status = format!("could not load spaces: {e:#}");
            false
        }
        Msg::Tasks(key, result) => {
            state.loading_tasks.remove(&key);
            match result {
                Ok(tasks) => {
                    state.stale.remove(&key);
                    state.tasks.insert(key, tasks);
                    state.done_refreshing();
                }
                Err(e) => state.status = format!("could not load tasks: {e:#}"),
            }
            false
        }
        Msg::Progress(m) => {
            state.status = m;
            false
        }
        Msg::Posted(result) => {
            state.status = result.unwrap_or_else(|e| format!("posting failed: {e:#}"));
            true
        }
        Msg::Implemented(task_id, result) => {
            state.implementing.remove(&task_id);
            state.status =
                result.unwrap_or_else(|e| format!("could not start the implementation: {e:#}"));
            true
        }
        Msg::Added(result) => {
            state.status = result.unwrap_or_else(|e| format!("could not add the repo: {e:#}"));
            true
        }
        Msg::Closed(result) => {
            state.status = result.unwrap_or_else(|e| format!("could not close the task: {e:#}"));
            true
        }
        Msg::Started(task_id, result) => {
            state.starting.remove(&task_id);
            state.status = match result {
                Ok(summary) => summary,
                Err(e) => format!("start failed: {e:#}"),
            };
            true
        }
        Msg::Detail(id, ticket, statuses) => {
            match ticket {
                Ok(ticket) => {
                    state.tickets.insert(id.clone(), *ticket);
                }
                Err(e) => state.status = format!("could not load comments: {e:#}"),
            }
            match statuses {
                Ok(statuses) => {
                    state.statuses.insert(id, statuses);
                }
                Err(e) => state.status = format!("could not load statuses: {e:#}"),
            }
            false
        }
        Msg::Teams(Ok(teams)) => {
            if teams.is_empty() {
                state.status = "this token can see no ClickUp workspace".into();
            } else {
                state.modal = Modal::Team {
                    choices: teams,
                    sel: 0,
                };
            }
            false
        }
        Msg::Teams(Err(e)) => {
            state.status = format!("could not load workspaces: {e:#}");
            false
        }
    }
}
