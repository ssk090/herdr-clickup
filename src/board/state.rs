//! The board's state machine: keys in, effects out. Drawing and threads live in `board`.
use crate::{
    app::{self, Mode},
    hierarchy::ListRef,
    hook::{Link, Update},
    ids::{PaneId, TaskId},
    task::{Task, Who},
    ticket::Ticket,
};
use ratatui::{
    crossterm::event::KeyCode,
    widgets::{ListState, TableState},
};

mod load;
mod modal;
mod nav;

pub use load::Msg;
pub use modal::Modal;
pub use nav::{Focus, RowKind, Source, SourceKey, task_key};
use std::collections::{BTreeMap, HashMap, HashSet};

pub const REFRESHING: &str = "refreshing…";

/// What the board does on the outside after a key. Everything else is state.
#[derive(Debug, PartialEq)]
pub enum Effect {
    LoadTasks {
        source: Source,
        closed: bool,
    },
    Refresh,
    Start {
        task: Task,
        mode: Mode,
        agent: String,
        repos: Vec<String>,
    },
    Implement {
        task: Task,
        pane: PaneId,
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
    ChooseTeam,
    SwitchTeam {
        id: String,
        name: String,
    },
}

pub struct BoardState {
    pub me: Option<u64>,
    pub spaces: Vec<(String, String)>,
    pub lists: Vec<ListRef>,
    pub loading_hierarchy: bool,
    pub expanded: HashSet<String>,
    pub side: ListState,
    pub source: Source,
    pub tasks: HashMap<SourceKey, Vec<Task>>,
    pub loading_tasks: HashSet<SourceKey>,
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
    pub links: HashMap<TaskId, (PaneId, Link)>,
    /// Drafted ClickUp updates by task id, waiting for approval.
    pub pending: HashMap<TaskId, Update>,
    /// Tickets by task id, for the comments in the detail view.
    pub tickets: HashMap<TaskId, Ticket>,
    /// The list's statuses by task id.
    pub statuses: HashMap<TaskId, Vec<String>>,
    /// The current source includes closed tasks.
    pub show_closed: bool,
    /// Tasks being started, so a second `s` cannot start a second worktree.
    pub starting: HashSet<TaskId>,
    /// Tasks whose apply workflow is being sent, so a second `i` cannot send it twice.
    pub implementing: HashSet<TaskId>,
    /// Sources whose data came from the cache and is not refreshed yet.
    pub stale: HashSet<SourceKey>,
    /// The hierarchy came from the cache and is not refreshed yet.
    pub stale_hierarchy: bool,
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
            show_closed: false,
            starting: HashSet::new(),
            implementing: HashSet::new(),
            stale: HashSet::new(),
            stale_hierarchy: false,
            quit: false,
        }
    }
}

impl BoardState {
    pub fn new() -> BoardState {
        BoardState::default()
    }

    pub fn receive(&mut self, msg: Msg) -> bool {
        load::receive(self, msg)
    }

    /// Clears the data of the old workspace, keeping the pane links.
    pub fn switch_team(&mut self) {
        self.spaces.clear();
        self.lists.clear();
        self.expanded.clear();
        self.tasks.clear();
        self.loading_tasks.clear();
        self.loading_hierarchy = false;
        self.me = None;
        self.source = Source::Mine;
        self.table = TableState::default();
        self.side.select(Some(0));
        self.focus = Focus::Side;
        self.stale.clear();
        self.stale_hierarchy = false;
        self.show_closed = false;
        self.tickets.clear();
        self.statuses.clear();
        self.filter.clear();
        self.filtering = false;
        self.who = Who::Anyone;
        self.who_label.clear();
    }

    pub fn set_links(&mut self, panes: BTreeMap<PaneId, Link>) {
        self.pending = panes
            .values()
            .filter_map(|l| Some((l.task_id.clone(), l.pending.clone()?)))
            .collect();
        self.links = panes
            .into_iter()
            .map(|(pane, link)| (link.task_id.clone(), (pane, link)))
            .collect();
    }

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
            return self.filter_key(code);
        }
        match std::mem::replace(&mut self.modal, Modal::None) {
            Modal::None => self.top_key(code),
            modal => modal::key(self, modal, code),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hook::{Phase, RepoWorktree},
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
            closed: false,
        }
    }

    fn state_with(tasks: Vec<Task>) -> BoardState {
        let mut s = BoardState::new();
        s.tasks.insert(SourceKey::Mine, tasks);
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
            vec![Effect::LoadTasks {
                source: Source::List(s.lists[0].clone()),
                closed: false,
            }]
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
        s.stale_hierarchy = true;
        s.stale.insert(SourceKey::Mine);
        s.receive(Msg::Hierarchy(Ok((vec![], vec![], None, 0))));
        s.receive(Msg::Tasks(SourceKey::Mine, Ok(vec![])));
        assert!(s.stale.is_empty());
        assert!(!s.stale_hierarchy);
    }

    #[test]
    fn receiving_tasks_clears_loading_and_hierarchy_opens_a_single_space() {
        let mut s = BoardState::new();
        s.loading_tasks.insert(SourceKey::Mine);
        s.receive(Msg::Tasks(SourceKey::Mine, Ok(vec![task("1", "one")])));
        assert!(!s.loading_tasks.contains(&SourceKey::Mine));
        assert_eq!(s.tasks[&SourceKey::Mine].len(), 1);
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

#[cfg(test)]
mod closed_tests {
    use super::*;

    #[test]
    fn closed_tasks_use_a_separate_key_and_load_when_missing() {
        let mut s = BoardState::new();
        s.tasks.insert(SourceKey::Mine, Vec::new());
        assert!(!s.show_closed);
        assert_eq!(
            s.on_key(KeyCode::Char('C')),
            vec![Effect::LoadTasks {
                source: Source::Mine,
                closed: true,
            }]
        );
        assert!(s.show_closed);
        s.tasks.insert(SourceKey::MineClosed, vec![]);
        assert!(s.on_key(KeyCode::Char('C')).is_empty());
        assert!(!s.show_closed);
    }
}

#[cfg(test)]
mod workspace_tests {
    use super::*;

    #[test]
    fn the_team_picker_switches_to_the_chosen_workspace() {
        let mut s = BoardState::new();
        assert_eq!(s.on_key(KeyCode::Char('W')), vec![Effect::ChooseTeam]);
        s.receive(Msg::Teams(Ok(vec![
            ("1".into(), "One".into()),
            ("2".into(), "Two".into()),
        ])));
        s.on_key(KeyCode::Down);
        assert_eq!(
            s.on_key(KeyCode::Enter),
            vec![Effect::SwitchTeam {
                id: "2".into(),
                name: "Two".into(),
            }]
        );
    }

    #[test]
    fn switching_teams_clears_the_old_data() {
        let mut s = BoardState::new();
        s.spaces = vec![("s".into(), "Space".into())];
        s.lists = vec![];
        s.tasks.insert(SourceKey::Mine, vec![]);
        s.show_closed = true;
        s.switch_team();
        assert!(s.spaces.is_empty());
        assert!(s.tasks.is_empty());
        assert!(!s.show_closed);
    }
}
