//! The board's state machine: keys in, effects out. Drawing and threads live in `board`.
use crate::{
    app::{self, Mode},
    hierarchy::{ListRef, current_sprints},
    hook::{Link, Phase, Update},
    task::{Task, Who, assignees_in, filter_tasks},
    ticket::Ticket,
};
use anyhow::Result;
use ratatui::{
    crossterm::event::KeyCode,
    widgets::{ListState, TableState},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

pub const REFRESHING: &str = "refreshing…";

/// Where the task table's tasks come from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    Mine,
    List(ListRef),
}

impl Source {
    pub fn key(&self) -> String {
        match self {
            Source::Mine => "mine".into(),
            Source::List(l) => l.id.clone(),
        }
    }

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

/// Spaces as `(id, name)`, all their lists, the token user's id, and how many spaces failed to load.
pub type Hierarchy = (Vec<(String, String)>, Vec<ListRef>, Option<u64>, usize);

pub enum Msg {
    Hierarchy(Result<Hierarchy>),
    Tasks(String, Result<Vec<Task>>),
    Started(String, Result<String>),
    /// Something the user must do for a start that is still running.
    Progress(String),
    /// An approved update was posted (or failed).
    Posted(Result<String>),
    /// The apply workflow was sent to an agent (or could not be).
    Implemented(String, Result<String>),
    /// A repo was added to a running task (or could not be).
    Added(Result<String>),
    /// A task was closed and its worktrees removed (or could not be).
    Closed(Result<String>),
    /// The ticket and the list's statuses for the detail view.
    Detail(String, Result<Box<Ticket>>, Result<Vec<String>>),
}

/// What the board does on the outside after a key. Everything else is state.
#[derive(Debug, PartialEq)]
pub enum Effect {
    LoadTasks(Source),
    Refresh,
    Start {
        task: Task,
        mode: Mode,
        agent: String,
        repos: Vec<String>,
    },
    Implement {
        task: Task,
        pane: String,
        change: String,
    },
    AddRepo {
        task: Task,
        repo: String,
    },
    ChooseAgent {
        task: Task,
        mode: Mode,
    },
    ChooseRepo {
        task: Task,
        mode: Mode,
        agent: String,
    },
    ChooseAddRepo(Task),
    PreviewClose(Task),
    CloseTask(Task),
    PostUpdate {
        task: Task,
        update: Update,
    },
    DiscardUpdate(Task),
    EditDraft {
        task: Task,
        update: Update,
    },
    OpenUrl(String),
    LoadDetail(Task),
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
}

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Side,
    Tasks,
}

pub struct BoardState {
    pub me: Option<u64>,
    pub spaces: Vec<(String, String)>,
    pub lists: Vec<ListRef>,
    pub loading_hierarchy: bool,
    pub expanded: HashSet<String>,
    pub side: ListState,
    pub source: Source,
    pub tasks: HashMap<String, Vec<Task>>,
    pub loading_tasks: HashSet<String>,
    pub table: TableState,
    pub focus: Focus,
    pub filter: String,
    pub filtering: bool,
    /// Whose tasks the table shows, and its label for the title (empty for everyone).
    pub who: Who,
    pub who_label: String,
    pub modal: Modal,
    pub status: String,
    /// Pane→task links by task id, reread from the state file: phase, branch and pane.
    pub links: HashMap<String, (String, Link)>,
    /// Drafted ClickUp updates by task id, waiting for approval.
    pub pending: HashMap<String, Update>,
    /// Tickets by task id, for the comments in the detail view.
    pub tickets: HashMap<String, Ticket>,
    /// The list's statuses by task id.
    pub statuses: HashMap<String, Vec<String>>,
    /// Tasks being started, so a second `s` cannot start a second worktree.
    pub starting: HashSet<String>,
    /// Tasks whose apply workflow is being sent, so a second `i` cannot send it twice.
    pub implementing: HashSet<String>,
    /// Sources whose data came from the cache and is not refreshed yet.
    pub stale: HashSet<String>,
    pub quit: bool,
}

impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            me: None,
            spaces: Vec::new(),
            lists: Vec::new(),
            loading_hierarchy: false,
            expanded: HashSet::new(),
            side: ListState::default().with_selected(Some(0)),
            source: Source::Mine,
            tasks: HashMap::new(),
            loading_tasks: HashSet::new(),
            table: TableState::default(),
            focus: Focus::Side,
            filter: String::new(),
            filtering: false,
            who: Who::Anyone,
            who_label: String::new(),
            modal: Modal::None,
            status: String::new(),
            links: HashMap::new(),
            pending: HashMap::new(),
            tickets: HashMap::new(),
            statuses: HashMap::new(),
            starting: HashSet::new(),
            implementing: HashSet::new(),
            stale: HashSet::new(),
            quit: false,
        }
    }
}

impl BoardState {
    pub fn new() -> BoardState {
        BoardState::default()
    }

    pub fn receive(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::Hierarchy(Ok((spaces, lists, me, failed))) => {
                self.loading_hierarchy = false;
                self.done_refreshing();
                if failed > 0 {
                    self.status =
                        format!("{failed} space(s) could not be loaded; press r to retry");
                }
                self.spaces = spaces;
                self.lists = lists;
                self.me = me;
                self.expand_single_space();
                self.stale.remove("hierarchy");
                false
            }
            Msg::Hierarchy(Err(e)) => {
                self.loading_hierarchy = false;
                self.status = format!("could not load spaces: {e:#}");
                false
            }
            Msg::Tasks(key, result) => {
                self.loading_tasks.remove(&key);
                match result {
                    Ok(tasks) => {
                        self.stale.remove(&key);
                        self.tasks.insert(key, tasks);
                        self.done_refreshing();
                    }
                    Err(e) => self.status = format!("could not load tasks: {e:#}"),
                }
                false
            }
            Msg::Progress(m) => {
                self.status = m;
                false
            }
            Msg::Posted(result) => {
                self.status = result.unwrap_or_else(|e| format!("posting failed: {e:#}"));
                true
            }
            Msg::Implemented(task_id, result) => {
                self.implementing.remove(&task_id);
                self.status =
                    result.unwrap_or_else(|e| format!("could not start the implementation: {e:#}"));
                true
            }
            Msg::Added(result) => {
                self.status = result.unwrap_or_else(|e| format!("could not add the repo: {e:#}"));
                true
            }
            Msg::Closed(result) => {
                self.status = result.unwrap_or_else(|e| format!("could not close the task: {e:#}"));
                true
            }
            Msg::Started(task_id, result) => {
                self.starting.remove(&task_id);
                self.status = match result {
                    Ok(summary) => summary,
                    Err(e) => format!("start failed: {e:#}"),
                };
                true
            }
            Msg::Detail(id, ticket, statuses) => {
                match ticket {
                    Ok(ticket) => {
                        self.tickets.insert(id.clone(), *ticket);
                    }
                    Err(e) => self.status = format!("could not load comments: {e:#}"),
                }
                match statuses {
                    Ok(statuses) => {
                        self.statuses.insert(id, statuses);
                    }
                    Err(e) => self.status = format!("could not load statuses: {e:#}"),
                }
                false
            }
        }
    }

    /// With a single space there is nothing to choose: open it.
    pub fn expand_single_space(&mut self) {
        if let [(_, name)] = self.spaces.as_slice() {
            self.expanded.insert(space_key(name));
        }
    }

    pub fn set_links(&mut self, panes: BTreeMap<String, Link>) {
        self.pending = panes
            .values()
            .filter_map(|l| Some((l.task_id.clone(), l.pending.clone()?)))
            .collect();
        self.links = panes
            .into_iter()
            .map(|(pane, link)| (link.task_id.clone(), (pane, link)))
            .collect();
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

    pub fn done_refreshing(&mut self) {
        if self.status == REFRESHING && !self.loading_hierarchy && self.loading_tasks.is_empty() {
            self.status.clear();
        }
    }

    // ---- derived views ----

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
            .get(&self.source.key())
            .map(Vec::as_slice)
            .unwrap_or_default();
        filter_tasks(all, &self.filter, self.who)
    }

    pub fn selected_task(&self) -> Option<Task> {
        self.table
            .selected()
            .and_then(|i| self.visible_tasks().get(i).map(|t| (*t).clone()))
    }

    pub fn phase_of(&self, task_id: &str) -> Option<Phase> {
        self.links.get(task_id).map(|(_, l)| l.phase())
    }

    // ---- modals opened by the runtime after it gathers choices ----

    pub fn open_agent(&mut self, task: Task, mode: Mode, choices: Vec<String>) {
        self.modal = Modal::Agent {
            task,
            mode,
            choices,
            sel: 0,
        };
    }

    pub fn open_repo(&mut self, task: Task, mode: Mode, agent: String, choices: Vec<String>) {
        let picked = vec![false; choices.len()];
        self.modal = Modal::Repo {
            task,
            mode,
            agent,
            choices,
            sel: 0,
            picked,
        };
    }

    pub fn open_add_repo(&mut self, task: Task, choices: Vec<String>) {
        self.modal = Modal::AddRepo {
            task,
            choices,
            sel: 0,
        };
    }

    pub fn open_close(&mut self, task: Task, preview: app::ClosePreview) {
        self.modal = Modal::Close { task, preview };
    }

    pub fn open_update(&mut self, task: Task, update: Update) {
        self.modal = Modal::Update { task, update };
    }

    // ---- input ----

    pub fn on_key(&mut self, code: KeyCode) -> Vec<Effect> {
        if self.filtering {
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
            return Vec::new();
        }
        match std::mem::replace(&mut self.modal, Modal::None) {
            Modal::None => self.top_key(code),
            Modal::Detail => self.detail_key(code),
            Modal::Agent {
                task,
                mode,
                choices,
                mut sel,
            } => match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    sel = sel.saturating_sub(1);
                    self.modal = Modal::Agent {
                        task,
                        mode,
                        choices,
                        sel,
                    };
                    Vec::new()
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    sel = (sel + 1).min(choices.len().saturating_sub(1));
                    self.modal = Modal::Agent {
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
                    self.modal = Modal::Agent {
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
                    self.modal = Modal::Update { task, update };
                    Vec::new()
                }
            },
            Modal::Who { choices, mut sel } => match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    sel = sel.saturating_sub(1);
                    self.modal = Modal::Who { choices, sel };
                    Vec::new()
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    sel = (sel + 1).min(choices.len().saturating_sub(1));
                    self.modal = Modal::Who { choices, sel };
                    Vec::new()
                }
                KeyCode::Enter => {
                    let (label, who) = choices[sel].clone();
                    // "jd (2)" -> "jd"; everyone needs no label.
                    let name = label
                        .rsplit_once(" (")
                        .map_or(label.as_str(), |(n, _)| n)
                        .to_string();
                    self.set_who(
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
                    self.modal = Modal::Who { choices, sel };
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
                    self.modal = Modal::Repo {
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
                    self.modal = Modal::Repo {
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
                    self.modal = Modal::Repo {
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
                    if !self.starting.insert(task.id.clone()) {
                        self.status = format!("{} is already starting", task.name);
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
                    self.modal = Modal::Repo {
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
                    self.modal = Modal::AddRepo { task, choices, sel };
                    Vec::new()
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    sel = (sel + 1).min(choices.len().saturating_sub(1));
                    self.modal = Modal::AddRepo { task, choices, sel };
                    Vec::new()
                }
                KeyCode::Enter => vec![Effect::AddRepo {
                    task,
                    repo: choices[sel].clone(),
                }],
                KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
                _ => {
                    self.modal = Modal::AddRepo { task, choices, sel };
                    Vec::new()
                }
            },
            Modal::Close { task, preview } => match code {
                KeyCode::Enter => vec![Effect::CloseTask(task)],
                KeyCode::Esc | KeyCode::Char('q') => Vec::new(),
                _ => {
                    self.modal = Modal::Close { task, preview };
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
                    self.modal = Modal::Status { task, choices, sel };
                    Vec::new()
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    sel = (sel + 1).min(choices.len().saturating_sub(1));
                    self.modal = Modal::Status { task, choices, sel };
                    Vec::new()
                }
                KeyCode::Enter => {
                    self.open_update(
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
                    self.modal = Modal::Status { task, choices, sel };
                    Vec::new()
                }
            },
        }
    }

    fn top_key(&mut self, code: KeyCode) -> Vec<Effect> {
        match code {
            KeyCode::Char('q') => {
                self.quit = true;
                Vec::new()
            }
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

    fn side_key(&mut self, code: KeyCode) -> Vec<Effect> {
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

    fn task_key(&mut self, code: KeyCode) -> Vec<Effect> {
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

    fn detail_key(&mut self, code: KeyCode) -> Vec<Effect> {
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

    fn show(&mut self, source: Source) -> Vec<Effect> {
        let mut effects = Vec::new();
        if !self.tasks.contains_key(&source.key()) {
            effects.push(Effect::LoadTasks(source.clone()));
        }
        self.source = source;
        self.table.select(Some(0));
        self.focus = Focus::Tasks;
        effects
    }

    fn choose_agent_effect(&mut self, mode: Mode) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        if self.starting.contains(&task.id) {
            self.status = format!("{} is already starting", task.name);
            return Vec::new();
        }
        vec![Effect::ChooseAgent { task, mode }]
    }

    fn implement_effect(&mut self) -> Vec<Effect> {
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

    fn add_repo_effect(&mut self) -> Vec<Effect> {
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

    fn close_effect(&mut self) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        vec![Effect::PreviewClose(task)]
    }

    fn status_effect(&mut self) -> Vec<Effect> {
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

    fn open_effect(&mut self) -> Vec<Effect> {
        let Some(task) = self.selected_task() else {
            return Vec::new();
        };
        if task.url.is_empty() {
            self.status = "this task has no ClickUp URL".into();
            return Vec::new();
        }
        vec![Effect::OpenUrl(task.url)]
    }

    fn review_update(&mut self) {
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
    fn choose_who(&mut self) {
        let all = self
            .tasks
            .get(&self.source.key())
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

/// The repos to start: the marked ones, or the highlighted repo when none are marked. This keeps
/// the old "move, Enter" single-repo behavior and lets space build a multi-repo selection.
pub fn chosen_repos(choices: &[String], picked: &[bool], sel: usize) -> Vec<String> {
    let mut chosen = picked_repos(choices, picked);
    if chosen.is_empty() {
        chosen.extend(choices.get(sel).cloned());
    }
    chosen
}

fn picked_repos(choices: &[String], picked: &[bool]) -> Vec<String> {
    choices
        .iter()
        .zip(picked)
        .filter(|(_, p)| **p)
        .map(|(c, _)| c.clone())
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hook::RepoWorktree,
        task::Assignee,
        ticket::{Comment, Ticket},
    };

    fn task(id: &str, name: &str) -> Task {
        Task {
            id: id.into(),
            name: name.into(),
            status: "to do".into(),
            url: format!("https://app.clickup.com/t/{id}"),
            description: String::new(),
            assignees: vec![],
            list_id: None,
        }
    }

    fn state_with(tasks: Vec<Task>) -> BoardState {
        let mut s = BoardState::new();
        s.tasks.insert("mine".into(), tasks);
        s.table.select(Some(0));
        s.focus = Focus::Tasks;
        s
    }

    fn spec_link() -> Link {
        Link {
            task_id: "1".into(),
            branch: "CU-1-fix".into(),
            change: Some("cu-1-fix".into()),
            phase: Some(Phase::SpecDrafted),
            dir: Some("/t/CU-1-fix".into()),
            worktrees: vec![RepoWorktree {
                repo: "/w/webui".into(),
                path: "/t/CU-1-fix/webui".into(),
            }],
            title: "Fix".into(),
            url: "u".into(),
            agent: "claude".into(),
            working: false,
            reported: true,
            pending: None,
        }
    }

    #[test]
    fn filtering_shows_matching_tasks_and_esc_clears_it() {
        let mut s = state_with(vec![task("1", "Fix login"), task("2", "Add button")]);
        assert_eq!(s.on_key(KeyCode::Char('/')), Vec::new());
        for c in "button".chars() {
            s.on_key(KeyCode::Char(c));
        }
        assert_eq!(s.visible_tasks().len(), 1);
        s.on_key(KeyCode::Esc);
        assert!(s.filter.is_empty());
        assert!(!s.filtering);
        assert_eq!(s.visible_tasks().len(), 2);
    }

    #[test]
    fn table_selection_stays_inside_the_visible_tasks() {
        let mut s = state_with(vec![task("1", "one"), task("2", "two")]);
        s.on_key(KeyCode::Down);
        assert_eq!(s.table.selected(), Some(1));
        s.on_key(KeyCode::Down);
        assert_eq!(s.table.selected(), Some(1));
        s.on_key(KeyCode::Up);
        s.on_key(KeyCode::Up);
        assert_eq!(s.table.selected(), Some(0));
    }

    #[test]
    fn expanding_a_space_reveals_its_lists_and_loads_a_list() {
        let mut s = BoardState::new();
        s.spaces = vec![("s1".into(), "Space".into())];
        s.lists = vec![ListRef {
            id: "l1".into(),
            name: "List".into(),
            space: "Space".into(),
            folder: None,
            start: None,
            due: None,
            task_count: None,
        }];
        s.side.select(Some(1));
        s.on_key(KeyCode::Enter);
        assert!(s.expanded.contains("s:Space"));
        s.side.select(Some(2));
        assert_eq!(
            s.on_key(KeyCode::Enter),
            vec![Effect::LoadTasks(Source::List(s.lists[0].clone()))]
        );
        assert_eq!(s.source, Source::List(s.lists[0].clone()));
    }

    #[test]
    fn move_then_enter_starts_the_highlighted_repo() {
        let mut s = state_with(vec![task("1", "one")]);
        s.open_repo(
            task("1", "one"),
            Mode::Code,
            "claude".into(),
            vec!["/a".into(), "/b".into()],
        );
        s.on_key(KeyCode::Down);
        assert_eq!(
            s.on_key(KeyCode::Enter),
            vec![Effect::Start {
                task: task("1", "one"),
                mode: Mode::Code,
                agent: "claude".into(),
                repos: vec!["/b".into()],
            }]
        );
    }

    #[test]
    fn space_marks_repos_for_a_multi_repo_start() {
        let mut s = state_with(vec![task("1", "one")]);
        s.open_repo(
            task("1", "one"),
            Mode::Spec,
            "claude".into(),
            vec!["/a".into(), "/b".into()],
        );
        s.on_key(KeyCode::Char(' '));
        s.on_key(KeyCode::Down);
        s.on_key(KeyCode::Char(' '));
        assert_eq!(
            s.on_key(KeyCode::Enter),
            vec![Effect::Start {
                task: task("1", "one"),
                mode: Mode::Spec,
                agent: "claude".into(),
                repos: vec!["/a".into(), "/b".into()],
            }]
        );
    }

    #[test]
    fn the_agent_modal_asks_for_repos_then_starts_once() {
        let mut s = state_with(vec![task("1", "one")]);
        s.open_agent(task("1", "one"), Mode::Code, vec!["claude".into()]);
        assert_eq!(
            s.on_key(KeyCode::Enter),
            vec![Effect::ChooseRepo {
                task: task("1", "one"),
                mode: Mode::Code,
                agent: "claude".into(),
            }]
        );
        s.open_repo(
            task("1", "one"),
            Mode::Code,
            "claude".into(),
            vec!["/a".into()],
        );
        s.on_key(KeyCode::Enter);
        s.open_repo(
            task("1", "one"),
            Mode::Code,
            "claude".into(),
            vec!["/a".into()],
        );
        assert!(s.on_key(KeyCode::Enter).is_empty());
        assert!(s.starting.contains("1"));
    }

    #[test]
    fn implement_is_offered_only_for_a_reviewed_change() {
        let mut s = state_with(vec![task("1", "one")]);
        assert!(s.on_key(KeyCode::Char('i')).is_empty());
        s.links.insert("1".into(), ("w1:p1".into(), spec_link()));
        assert_eq!(
            s.on_key(KeyCode::Char('i')),
            vec![Effect::Implement {
                task: task("1", "one"),
                pane: "w1:p1".into(),
                change: "cu-1-fix".into(),
            }]
        );
        assert!(s.implementing.contains("1"));
    }

    #[test]
    fn the_update_modal_posts_discards_and_edits() {
        let mut s = state_with(vec![task("1", "one")]);
        let update = Update {
            comment: "done".into(),
            status: Some("review".into()),
        };
        s.pending.insert("1".into(), update.clone());
        s.on_key(KeyCode::Char('u'));
        assert_eq!(
            s.on_key(KeyCode::Enter),
            vec![Effect::PostUpdate {
                task: task("1", "one"),
                update: update.clone(),
            }]
        );
        s.pending.insert("1".into(), update.clone());
        s.on_key(KeyCode::Char('u'));
        assert_eq!(
            s.on_key(KeyCode::Char('d')),
            vec![Effect::DiscardUpdate(task("1", "one"))]
        );
        s.pending.insert("1".into(), update.clone());
        s.on_key(KeyCode::Char('u'));
        assert_eq!(
            s.on_key(KeyCode::Char('e')),
            vec![Effect::EditDraft {
                task: task("1", "one"),
                update,
            }]
        );
    }

    #[test]
    fn the_detail_and_table_actions_emit_their_effects() {
        let mut s = state_with(vec![task("1", "one")]);
        assert_eq!(
            s.on_key(KeyCode::Char('s')),
            vec![Effect::ChooseAgent {
                task: task("1", "one"),
                mode: Mode::Spec,
            }]
        );
        assert_eq!(
            s.on_key(KeyCode::Char('c')),
            vec![Effect::ChooseAgent {
                task: task("1", "one"),
                mode: Mode::Code,
            }]
        );
        assert_eq!(
            s.on_key(KeyCode::Char('X')),
            vec![Effect::PreviewClose(task("1", "one"))]
        );
        assert_eq!(
            s.on_key(KeyCode::Char('o')),
            vec![Effect::OpenUrl("https://app.clickup.com/t/1".into())]
        );
        s.on_key(KeyCode::Enter);
        assert!(matches!(s.modal, Modal::Detail));
        assert_eq!(
            s.on_key(KeyCode::Char('X')),
            vec![Effect::PreviewClose(task("1", "one"))]
        );
    }

    #[test]
    fn the_detail_loads_comments_and_a_status_change_makes_a_draft() {
        let mut s = state_with(vec![task("1", "one")]);
        assert_eq!(
            s.on_key(KeyCode::Enter),
            vec![Effect::LoadDetail(task("1", "one"))]
        );
        let ticket = Ticket {
            task: task("1", "one"),
            tags: Vec::new(),
            parent: None,
            custom_fields: Vec::new(),
            checklists: Vec::new(),
            subtasks: Vec::new(),
            attachments: Vec::new(),
            comments: vec![Comment {
                user: "Deepti".into(),
                date_ms: 0,
                text: "Retry 3 times.".into(),
            }],
        };
        s.receive(Msg::Detail(
            "1".into(),
            Ok(Box::new(ticket)),
            Ok(vec!["to do".into(), "review".into()]),
        ));
        assert!(s.tickets.contains_key("1"));
        assert!(s.on_key(KeyCode::Char('t')).is_empty());
        assert!(matches!(s.modal, Modal::Status { .. }));
        s.on_key(KeyCode::Down);
        assert!(s.on_key(KeyCode::Enter).is_empty());
        match &s.modal {
            Modal::Update { update, .. } => {
                assert!(update.comment.is_empty());
                assert_eq!(update.status.as_deref(), Some("review"));
            }
            _ => panic!("expected a status draft"),
        }
    }

    #[test]
    fn mine_toggles_who_and_assignee_choices_count_everyone() {
        let mut s = state_with(vec![Task {
            assignees: vec![Assignee {
                id: 7,
                username: "Deepti".into(),
            }],
            ..task("1", "one")
        }]);
        s.me = Some(7);
        s.on_key(KeyCode::Char('m'));
        assert_eq!(s.who, Who::Person(7));
        assert_eq!(s.who_label, "me");
        s.on_key(KeyCode::Char('m'));
        assert_eq!(s.who, Who::Anyone);
        s.on_key(KeyCode::Char('a'));
        match &s.modal {
            Modal::Who { choices, .. } => {
                assert_eq!(choices[0].0, "Everyone (1)");
                assert_eq!(choices[1].0, "Me (1)");
            }
            _ => panic!("expected the who modal"),
        }
    }

    #[test]
    fn a_refreshed_source_leaves_the_cached_mark() {
        let mut s = BoardState::new();
        s.stale.insert("hierarchy".into());
        s.stale.insert("mine".into());
        s.receive(Msg::Hierarchy(Ok((vec![], vec![], None, 0))));
        s.receive(Msg::Tasks("mine".into(), Ok(vec![])));
        assert!(s.stale.is_empty());
    }

    #[test]
    fn receiving_tasks_clears_loading_and_hierarchy_opens_a_single_space() {
        let mut s = BoardState::new();
        s.loading_tasks.insert("mine".into());
        s.receive(Msg::Tasks("mine".into(), Ok(vec![task("1", "one")])));
        assert!(!s.loading_tasks.contains("mine"));
        assert_eq!(s.tasks["mine"].len(), 1);
        s.receive(Msg::Hierarchy(Ok((
            vec![("s1".into(), "Space".into())],
            vec![],
            Some(7),
            0,
        ))));
        assert_eq!(s.me, Some(7));
        assert!(s.expanded.contains("s:Space"));
    }
}
