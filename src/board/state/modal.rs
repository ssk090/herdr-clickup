//! The board modals: the state of a dialog, and the keys it takes.
use super::{BoardState, Effect, chosen_repos};
use crate::{
    app::{self, Mode},
    hook::Update,
    task::{Task, Who},
};
use ratatui::crossterm::event::KeyCode;

pub enum Modal {
    None,
    Detail,
    /// Holds the task itself: the table can reload while the modal is open.
    Agent {
        task: Task,
        mode: Mode,
        choices: Vec<String>,
        sel: usize,
    },
    /// A drafted ClickUp update, shown in full; posted only on Enter.
    Update {
        task: Task,
        update: Update,
    },
    /// Whose tasks to show: everyone, me, unassigned, or one person.
    Who {
        choices: Vec<(String, Who)>,
        sel: usize,
    },
    /// Second step of a start: the agent is chosen, now the repos (space toggles, first hosts a spec).
    Repo {
        task: Task,
        mode: Mode,
        agent: String,
        choices: Vec<String>,
        sel: usize,
        picked: Vec<bool>,
    },
    /// Add one more repo to a running multi-repo task.
    AddRepo {
        task: Task,
        choices: Vec<String>,
        sel: usize,
    },
    /// Close a task: remove every worktree and its task folder.
    Close {
        task: Task,
        preview: app::ClosePreview,
    },
    /// Pick a status from the list's statuses.
    Status {
        task: Task,
        choices: Vec<String>,
        sel: usize,
    },
    /// Pick a ClickUp workspace.
    Team {
        choices: Vec<(String, String)>,
        sel: usize,
    },
}

pub(super) fn key(state: &mut BoardState, modal: Modal, code: KeyCode) -> Vec<Effect> {
    match modal {
        Modal::None => Vec::new(),
        Modal::Detail => state.detail_key(code),
        Modal::Agent {
            task,
            mode,
            choices,
            mut sel,
        } => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                sel = sel.saturating_sub(1);
                state.modal = Modal::Agent {
                    task,
                    mode,
                    choices,
                    sel,
                };
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                sel = (sel + 1).min(choices.len().saturating_sub(1));
                state.modal = Modal::Agent {
                    task,
                    mode,
                    choices,
                    sel,
                };
                Vec::new()
            }
            KeyCode::Enter => vec![Effect::ChooseRepo {
                task,
                mode,
                agent: choices[sel].clone(),
            }],
            KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
            _ => {
                state.modal = Modal::Agent {
                    task,
                    mode,
                    choices,
                    sel,
                };
                Vec::new()
            }
        },
        Modal::Update { task, update } => match code {
            KeyCode::Char('e') => vec![Effect::EditDraft { task, update }],
            KeyCode::Enter => vec![Effect::PostUpdate { task, update }],
            KeyCode::Char('d') => vec![Effect::DiscardUpdate(task)],
            KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
            _ => {
                state.modal = Modal::Update { task, update };
                Vec::new()
            }
        },
        Modal::Who { choices, mut sel } => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                sel = sel.saturating_sub(1);
                state.modal = Modal::Who { choices, sel };
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                sel = (sel + 1).min(choices.len().saturating_sub(1));
                state.modal = Modal::Who { choices, sel };
                Vec::new()
            }
            KeyCode::Enter => {
                let (label, who) = choices[sel].clone();
                // "jd (2)" -> "jd"; everyone needs no label.
                let name = label
                    .rsplit_once(" (")
                    .map_or(label.as_str(), |(n, _)| n)
                    .to_string();
                state.set_who(
                    who,
                    if who == Who::Anyone {
                        String::new()
                    } else {
                        name
                    },
                );
                Vec::new()
            }
            KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
            _ => {
                state.modal = Modal::Who { choices, sel };
                Vec::new()
            }
        },
        Modal::Repo {
            task,
            mode,
            agent,
            choices,
            mut sel,
            mut picked,
        } => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                sel = sel.saturating_sub(1);
                state.modal = Modal::Repo {
                    task,
                    mode,
                    agent,
                    choices,
                    sel,
                    picked,
                };
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                sel = (sel + 1).min(choices.len().saturating_sub(1));
                state.modal = Modal::Repo {
                    task,
                    mode,
                    agent,
                    choices,
                    sel,
                    picked,
                };
                Vec::new()
            }
            KeyCode::Char(' ') => {
                if let Some(p) = picked.get_mut(sel) {
                    *p = !*p;
                }
                state.modal = Modal::Repo {
                    task,
                    mode,
                    agent,
                    choices,
                    sel,
                    picked,
                };
                Vec::new()
            }
            KeyCode::Enter => {
                let repos = chosen_repos(&choices, &picked, sel);
                if !state.starting.insert(task.id.clone()) {
                    state.status = format!("{} is already starting", task.name);
                    Vec::new()
                } else {
                    vec![Effect::Start {
                        task,
                        mode,
                        agent,
                        repos,
                    }]
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
            _ => {
                state.modal = Modal::Repo {
                    task,
                    mode,
                    agent,
                    choices,
                    sel,
                    picked,
                };
                Vec::new()
            }
        },
        Modal::AddRepo {
            task,
            choices,
            mut sel,
        } => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                sel = sel.saturating_sub(1);
                state.modal = Modal::AddRepo { task, choices, sel };
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                sel = (sel + 1).min(choices.len().saturating_sub(1));
                state.modal = Modal::AddRepo { task, choices, sel };
                Vec::new()
            }
            KeyCode::Enter => vec![Effect::AddRepo {
                task,
                repo: choices[sel].clone(),
            }],
            KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
            _ => {
                state.modal = Modal::AddRepo { task, choices, sel };
                Vec::new()
            }
        },
        Modal::Close { task, preview } => match code {
            KeyCode::Enter => vec![Effect::CloseTask(task)],
            KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
            _ => {
                state.modal = Modal::Close { task, preview };
                Vec::new()
            }
        },
        Modal::Team { choices, mut sel } => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                sel = sel.saturating_sub(1);
                state.modal = Modal::Team { choices, sel };
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                sel = (sel + 1).min(choices.len().saturating_sub(1));
                state.modal = Modal::Team { choices, sel };
                Vec::new()
            }
            KeyCode::Enter => {
                let (id, name) = choices[sel].clone();
                vec![Effect::SwitchTeam { id, name }]
            }
            KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
            _ => {
                state.modal = Modal::Team { choices, sel };
                Vec::new()
            }
        },
        Modal::Status {
            task,
            choices,
            mut sel,
        } => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                sel = sel.saturating_sub(1);
                state.modal = Modal::Status { task, choices, sel };
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                sel = (sel + 1).min(choices.len().saturating_sub(1));
                state.modal = Modal::Status { task, choices, sel };
                Vec::new()
            }
            KeyCode::Enter => {
                state.open_update(
                    task,
                    Update {
                        comment: String::new(),
                        status: Some(choices[sel].clone()),
                    },
                );
                Vec::new()
            }
            KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
            _ => {
                state.modal = Modal::Status { task, choices, sel };
                Vec::new()
            }
        },
    }
}
