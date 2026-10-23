//! Navigation and filtering: the side tree, the task table, and the keys that move them.
use super::{BoardState, Effect, Modal, REFRESHING};
use crate::{
    app::Mode,
    hierarchy::{ListRef, current_sprints},
    hook::Phase,
    ids::{ListId, TaskId},
    task::{Task, Who, assignees_in, filter_tasks},
};
use ratatui::crossterm::event::KeyCode;
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

/// Where the task table's tasks come from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    Mine,
    List(ListRef),
}

impl Source {
    pub fn title(&self) -> String {
        match self {
            Source::Mine => "My tasks".into(),
            Source::List(l) => match &l.folder {
                Some(f) => format!("{} · {f} / {}", l.space, l.name),
                None => format!("{} / {}", l.space, l.name),
            },
        }
    }
}

/// The typed key of a task source, with the closed flag. No source carries a magic string.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SourceKey {
    Mine,
    List(ListId),
    MineClosed,
    ListClosed(ListId),
}

impl SourceKey {
    pub fn of(source: &Source, closed: bool) -> Self {
        match (source, closed) {
            (Source::Mine, false) => SourceKey::Mine,
            (Source::Mine, true) => SourceKey::MineClosed,
            (Source::List(l), false) => SourceKey::List(l.id.clone()),
            (Source::List(l), true) => SourceKey::ListClosed(l.id.clone()),
        }
    }

    /// The key for the cache file: `mine`, a list id, or the same with `#closed`.
    pub fn cache_key(&self) -> String {
        match self {
            SourceKey::Mine => "mine".into(),
            SourceKey::MineClosed => "mine#closed".into(),
            SourceKey::List(id) => id.to_string(),
            SourceKey::ListClosed(id) => format!("{id}#closed"),
        }
    }
}

#[derive(Clone)]
pub enum RowKind {
    Mine,
    Header,
    Space(String),
    Folder(String),
    List(ListRef),
}

pub struct SideRow {
    pub depth: usize,
    pub label: String,
    pub kind: RowKind,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Side,
    Tasks,
}

impl BoardState {
    pub(super) fn filter_key(&mut self, code: KeyCode) -> Vec<Effect> {
        match code {
            KeyCode::Esc => {
                self.filter.clear();
                self.filtering = false;
            }
            KeyCode::Enter => self.filtering = false,
            KeyCode::Backspace => drop(self.filter.pop()),
            KeyCode::Char(c) => self.filter.push(c),
            _ => {}
        }
        self.table.select(Some(0));
        Vec::new()
    }

    /// With a single space there is nothing to choose: open it.
    pub fn expand_single_space(&mut self) {
        if let [(_, name)] = self.spaces.as_slice() {
            self.expanded.insert(space_key(name));
        }
    }

    pub fn set_who(&mut self, who: Who, label: String) {
        self.who = who;
        self.who_label = label;
        self.table.select(Some(0));
    }

    /// True when the selected task has a reviewed spec that is ready to implement.
    pub fn can_implement(&self) -> bool {
        self.selected_task().is_some_and(|t| {
            self.links
                .get(&t.id)
                .is_some_and(|(_, l)| l.can_implement())
        })
    }

    pub fn side_rows(&self) -> Vec<SideRow> {
        let mut rows = vec![SideRow {
            depth: 0,
            label: "★ My tasks".into(),
            kind: RowKind::Mine,
        }];
        let sprints = current_sprints(&self.lists, now_ms());
        if !sprints.is_empty() {
            rows.push(SideRow {
                depth: 0,
                label: "Current sprints".into(),
                kind: RowKind::Header,
            });
            for l in sprints {
                rows.push(SideRow {
                    depth: 1,
                    label: list_label(l),
                    kind: RowKind::List(l.clone()),
                });
            }
        }
        for (_, space) in &self.spaces {
            let open = self.expanded.contains(&space_key(space));
            rows.push(SideRow {
                depth: 0,
                label: format!("{} {space}", arrow(open)),
                kind: RowKind::Space(space.clone()),
            });
            if !open {
                continue;
            }
            let mut folders: Vec<&str> = Vec::new();
            for l in self.lists.iter().filter(|l| &l.space == space) {
                match l.folder.as_deref() {
                    Some(f) if !folders.contains(&f) => folders.push(f),
                    Some(_) => {}
                    None => rows.push(SideRow {
                        depth: 1,
                        label: list_label(l),
                        kind: RowKind::List(l.clone()),
                    }),
                }
            }
            for f in folders {
                let key = folder_key(space, f);
                let open = self.expanded.contains(&key);
                rows.push(SideRow {
                    depth: 1,
                    label: format!("{} {f}", arrow(open)),
                    kind: RowKind::Folder(key),
                });
                if open {
                    for l in self
                        .lists
                        .iter()
                        .filter(|l| &l.space == space && l.folder.as_deref() == Some(f))
                    {
                        rows.push(SideRow {
                            depth: 2,
                            label: list_label(l),
                            kind: RowKind::List(l.clone()),
                        });
                    }
                }
            }
        }
        rows
    }

    pub fn visible_tasks(&self) -> Vec<&Task> {
        let all = self
            .tasks
            .get(&task_key(&self.source, self.show_closed))
            .map(Vec::as_slice)
            .unwrap_or_default();
        filter_tasks(all, &self.filter, self.who)
    }

    pub fn selected_task(&self) -> Option<Task> {
        self.table
            .selected()
            .and_then(|i| self.visible_tasks().get(i).map(|t| (*t).clone()))
    }

    pub fn phase_of(&self, task_id: &TaskId) -> Option<Phase> {
        self.links.get(task_id).map(|(_, l)| l.phase())
    }

    // ---- modals opened by the runtime after it gathers choices ----

    pub(super) fn top_key(&mut self, code: KeyCode) -> Vec<Effect> {
        match code {
            KeyCode::Char('q') => {
                self.quit = true;
                Vec::new()
            }
            KeyCode::Char('W') => vec![Effect::ChooseTeam],
            // Esc is too easy to hit to close the whole board; it only clears the filter.
            KeyCode::Esc => {
                self.filter.clear();
                self.status.clear();
                self.table.select(Some(0));
                Vec::new()
            }
            KeyCode::Tab => {
                self.focus = if self.focus == Focus::Side {
                    Focus::Tasks
                } else {
                    Focus::Side
                };
                Vec::new()
            }
            KeyCode::Char('/') => {
                self.filtering = true;
                self.focus = Focus::Tasks;
                Vec::new()
            }
            KeyCode::Char('C') => {
                self.show_closed = !self.show_closed;
                self.table.select(Some(0));
                let key = task_key(&self.source, self.show_closed);
                if self.tasks.contains_key(&key) {
                    Vec::new()
                } else {
                    vec![Effect::LoadTasks {
                        source: self.source.clone(),
                        closed: self.show_closed,
                    }]
                }
            }
            KeyCode::Char('m') if self.me.is_none() => {
                self.status =
                    "your ClickUp user is not loaded yet; try again in a moment (r reloads)".into();
                Vec::new()
            }
            KeyCode::Char('m') => {
                match (self.who, self.me) {
                    (Who::Person(id), Some(me)) if id == me => {
                        self.set_who(Who::Anyone, String::new())
                    }
                    (_, Some(me)) => self.set_who(Who::Person(me), "me".into()),
                    (_, None) => {}
                }
                Vec::new()
            }
            KeyCode::Char('a') => {
                self.choose_who();
                Vec::new()
            }
            // The old tasks stay on screen until the new ones arrive, so a failed reload loses nothing.
            KeyCode::Char('r') => {
                self.status = REFRESHING.into();
                vec![Effect::Refresh]
            }
            _ if self.focus == Focus::Side => self.side_key(code),
            _ => self.task_key(code),
        }
    }

    pub(super) fn side_key(&mut self, code: KeyCode) -> Vec<Effect> {
        let rows = self.side_rows();
        if rows.is_empty() {
            return Vec::new();
        }
        let sel = self
            .side
            .selected()
            .unwrap_or(0)
            .min(rows.len().saturating_sub(1));
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.side.select(Some(sel.saturating_sub(1)));
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.side
                    .select(Some((sel + 1).min(rows.len().saturating_sub(1))));
                Vec::new()
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => match rows[sel].kind.clone() {
                RowKind::Space(s) => {
                    toggle(&mut self.expanded, space_key(&s));
                    Vec::new()
                }
                RowKind::Folder(key) => {
                    toggle(&mut self.expanded, key);
                    Vec::new()
                }
                RowKind::Mine => self.show(Source::Mine),
                RowKind::List(l) => self.show(Source::List(l)),
                RowKind::Header => Vec::new(),
            },
            KeyCode::Left | KeyCode::Char('h') => {
                match &rows[sel].kind {
                    RowKind::Space(s) => drop(self.expanded.remove(&space_key(s))),
                    RowKind::Folder(key) => drop(self.expanded.remove(key)),
                    _ => {}
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    pub(super) fn task_key(&mut self, code: KeyCode) -> Vec<Effect> {
        let n = self.visible_tasks().len();
        let sel = self.table.selected().unwrap_or(0);
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.table.select(Some(sel.saturating_sub(1)));
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.table.select(Some((sel + 1).min(n.saturating_sub(1))));
                Vec::new()
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = Focus::Side;
                Vec::new()
            }
            KeyCode::Enter if n > 0 => {
                self.modal = Modal::Detail;
                let Some(task) = self.selected_task() else {
                    return Vec::new();
                };
                vec![Effect::LoadDetail(task)]
            }
            KeyCode::Char('t') if n > 0 => self.status_effect(),
            KeyCode::Char('s') if n > 0 => self.choose_agent_effect(Mode::Spec),
            KeyCode::Char('i') if n > 0 => self.implement_effect(),
            KeyCode::Char('c') if n > 0 => self.choose_agent_effect(Mode::Code),
            KeyCode::Char('A') if n > 0 => self.add_repo_effect(),
            KeyCode::Char('X') if n > 0 => self.close_effect(),
            KeyCode::Char('u') => {
                self.review_update();
                Vec::new()
            }
            KeyCode::Char('o') => self.open_effect(),
            _ => Vec::new(),
        }
    }

    pub(super) fn detail_key(&mut self, code: KeyCode) -> Vec<Effect> {
        match code {
            KeyCode::Char('s') => self.choose_agent_effect(Mode::Spec),
            KeyCode::Char('i') => self.implement_effect(),
            KeyCode::Char('c') => self.choose_agent_effect(Mode::Code),
            KeyCode::Char('A') => self.add_repo_effect(),
            KeyCode::Char('X') => self.close_effect(),
            KeyCode::Char('o') => self.open_effect(),
            KeyCode::Char('t') => self.status_effect(),
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => Vec::new(),
            _ => {
                self.modal = Modal::Detail;
                Vec::new()
            }
        }
    }

    pub(super) fn show(&mut self, source: Source) -> Vec<Effect> {
        let mut effects = Vec::new();
        if !self
            .tasks
            .contains_key(&task_key(&source, self.show_closed))
        {
            effects.push(Effect::LoadTasks {
                source: source.clone(),
                closed: self.show_closed,
            });
        }
        self.source = source;
        self.table.select(Some(0));
        self.focus = Focus::Tasks;
        effects
    }

    pub(super) fn choose_agent_effect(&mut self, mode: Mode) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        if self.starting.contains(&task.id) {
            self.status = format!("{} is already starting", task.name);
            return Vec::new();
        }
        vec![Effect::ChooseAgent { task, mode }]
    }

    pub(super) fn implement_effect(&mut self) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        let Some((pane, link)) = self.links.get(&task.id).cloned() else {
            self.status =
                "no OpenSpec change for this task; press s to write the spec first".into();
            return Vec::new();
        };
        let Some(change) = link.change.clone() else {
            self.status = "this is a code start; there is no OpenSpec change to implement".into();
            return Vec::new();
        };
        if !link.can_implement() {
            self.status = format!(
                "{} cannot be implemented yet ({})",
                task.name,
                link.phase().label()
            );
            return Vec::new();
        }
        if !self.implementing.insert(task.id.clone()) {
            self.status = format!("{} is already starting the implementation", task.name);
            return Vec::new();
        }
        self.status = format!("sending the apply workflow for {change}…");
        vec![Effect::Implement { task, pane, change }]
    }

    pub(super) fn add_repo_effect(&mut self) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        let Some((_, link)) = self.links.get(&task.id).cloned() else {
            self.status = "this task has no running worktree".into();
            return Vec::new();
        };
        if link.folder().is_none() {
            self.status = "only a task started with 2+ repos has a task folder".into();
            return Vec::new();
        }
        vec![Effect::ChooseAddRepo(task)]
    }

    pub(super) fn close_effect(&mut self) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        vec![Effect::PreviewClose(task)]
    }

    pub(super) fn status_effect(&mut self) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        match self.statuses.get(&task.id).cloned() {
            Some(choices) if !choices.is_empty() => {
                self.modal = Modal::Status {
                    task,
                    choices,
                    sel: 0,
                };
                Vec::new()
            }
            Some(_) => {
                self.status = "this list has no statuses".into();
                self.modal = Modal::Detail;
                Vec::new()
            }
            None => {
                self.status = "loading the list's statuses…".into();
                self.modal = Modal::Detail;
                vec![Effect::LoadDetail(task)]
            }
        }
    }

    pub(super) fn open_effect(&mut self) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        if task.url.is_empty() {
            self.status = "this task has no ClickUp URL".into();
            return Vec::new();
        }
        vec![Effect::OpenUrl(task.url)]
    }

    pub(super) fn review_update(&mut self) {
        let Some(task) = self.selected_task() else {
            return;
        };
        match self.pending.get(&task.id).cloned() {
            Some(update) => self.modal = Modal::Update { task, update },
            None => {
                self.status =
                    "no drafted ClickUp update for this task (✉ marks the ones with one)".into()
            }
        }
    }

    /// Everyone, me, unassigned, then each person in the current list's tasks with their task count.
    pub(super) fn choose_who(&mut self) {
        let all = self
            .tasks
            .get(&task_key(&self.source, self.show_closed))
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut choices = vec![(format!("Everyone ({})", all.len()), Who::Anyone)];
        if let Some(me) = self.me {
            choices.push((
                format!(
                    "Me ({})",
                    all.iter().filter(|t| t.is_assigned_to(me)).count()
                ),
                Who::Person(me),
            ));
        }
        choices.push((
            format!(
                "Unassigned ({})",
                all.iter().filter(|t| t.assignees.is_empty()).count()
            ),
            Who::Nobody,
        ));
        for (a, n) in assignees_in(all) {
            if Some(a.id) != self.me {
                let name = if a.username.is_empty() {
                    format!("user {}", a.id)
                } else {
                    a.username
                };
                choices.push((format!("{name} ({n})"), Who::Person(a.id)));
            }
        }
        let sel = choices
            .iter()
            .position(|(_, w)| *w == self.who)
            .unwrap_or(0);
        self.modal = Modal::Who { choices, sel };
    }
}

/// The cache and task map key for a source, with or without closed tasks.
pub fn task_key(source: &Source, closed: bool) -> SourceKey {
    SourceKey::of(source, closed)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

fn space_key(space: &str) -> String {
    format!("s:{space}")
}

fn folder_key(space: &str, folder: &str) -> String {
    format!("f:{space}/{folder}")
}

fn arrow(open: bool) -> &'static str {
    if open { "▾" } else { "▸" }
}

fn list_label(l: &ListRef) -> String {
    match l.task_count {
        Some(n) => format!("{}  ({n})", l.name),
        None => l.name.clone(),
    }
}

fn toggle(set: &mut HashSet<String>, key: String) {
    if !set.remove(&key) {
        set.insert(key);
    }
}
