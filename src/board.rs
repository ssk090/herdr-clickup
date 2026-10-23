//! The ClickUp board: a herdr tab with sprints and lists on the left and their tasks on the right.
//! `state` holds the keys, modals and selections. This module draws them and runs the threads.
mod state;

use crate::{
    app::{self, Mode, NO_AGENT, NO_REPO},
    cache,
    clickup::ClickUp,
    config::Config,
    herdr,
    hook::{Phase, Update},
    store,
    task::Task,
    taskspace,
};
use anyhow::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
};
use state::{BoardState, Effect, Focus, Modal, Msg, REFRESHING, RowKind, Source, task_key};
use std::{
    fs,
    process::Command,
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::{Duration, Instant},
};

struct Board {
    cfg: Config,
    cu: ClickUp,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    state: BoardState,
    pending_read: Instant,
    /// Set by `e`: the board leaves the TUI, runs the editor, then reopens the update dialog.
    edit_request: Option<(Task, Update)>,
}

pub fn run() -> Result<()> {
    // Herdr does not replay close events, so drop links to panes that are already gone.
    if let Ok(live) = herdr::live_panes() {
        let _ = store::prune(&live);
    }
    let (cfg, cu) = app::client()?;
    let mut terminal = ratatui::init();
    let result = Board::new(cfg, cu).run(&mut terminal);
    ratatui::restore();
    result
}

impl Board {
    fn new(cfg: Config, cu: ClickUp) -> Board {
        let (tx, rx) = channel();
        let mut board = Board {
            cfg,
            cu,
            tx,
            rx,
            state: BoardState::new(),
            pending_read: Instant::now() - Duration::from_secs(60),
            edit_request: None,
        };
        // Show the last data at once, then refresh it in the background.
        if let Some(dir) = cache::state_dir() {
            if let Some(h) = cache::read_hierarchy(&dir) {
                board.state.spaces = h.spaces;
                board.state.lists = h.lists;
                board.state.me = h.me;
                board.state.expand_single_space();
                board.state.stale.insert("hierarchy".into());
            }
            if let Some(tasks) = cache::read_tasks(&dir, "mine") {
                board.state.tasks.insert("mine".into(), tasks);
                board.state.stale.insert("mine".into());
            }
        }
        board.load_hierarchy();
        board.load_tasks(Source::Mine, false);
        board
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.state.quit {
            if self.pending_read.elapsed() > Duration::from_secs(1) {
                self.read_state();
            }
            terminal.draw(|f| self.draw(f))?;
            while let Ok(msg) = self.rx.try_recv() {
                self.receive(msg);
            }
            if event::poll(Duration::from_millis(100))?
                && let Event::Key(k) = event::read()?
                && k.kind == KeyEventKind::Press
            {
                for effect in self.state.on_key(k.code) {
                    self.apply(effect);
                }
                if let Some((task, update)) = self.edit_request.take() {
                    let edited = self.edit(&task, &update);
                    // The editor ran on the normal screen, so the TUI starts again.
                    *terminal = ratatui::init();
                    match edited {
                        Some(update) => {
                            self.state.status = match app::save_pending(&task.id, &update) {
                                Ok(()) => "saved the edited draft".into(),
                                Err(e) => format!("could not save the edited draft: {e:#}"),
                            };
                            self.read_state();
                            self.state.open_update(task, update);
                        }
                        None => {
                            self.state.status = "the editor closed without a saved draft".into();
                            self.state.open_update(task, update);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Opens `$EDITOR` on the draft, on the normal screen. The run loop restores the TUI after.
    fn edit(&self, task: &Task, update: &Update) -> Option<Update> {
        // The caller always re-enters the TUI, even when the file cannot be written.
        ratatui::restore();
        let path = std::env::temp_dir().join(format!("herdr-clickup-{}.md", task.id));
        fs::write(&path, app::edit_text(update)).ok()?;
        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .ok()
            .filter(|e| !e.trim().is_empty())
            .unwrap_or_else(|| "vi".into());
        let quoted = format!("'{}'", path.display().to_string().replace('\'', "'\\''"));
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!("{editor} {quoted}"))
            .status();
        let text = fs::read_to_string(&path).ok();
        let _ = fs::remove_file(&path);
        match (status, text) {
            (Ok(s), Some(t)) if s.success() => Some(app::parse_edit(&t)),
            _ => None,
        }
    }

    // ---- loading ----

    fn load_hierarchy(&mut self) {
        if self.state.loading_hierarchy {
            return;
        }
        self.state.loading_hierarchy = true;
        let (cu, only, tx) = (self.cu.clone(), self.cfg.space_ids.clone(), self.tx.clone());
        thread::spawn(move || {
            let result = (|| {
                let spaces = cu.spaces(&only)?;
                // At most four spaces at a time, so a burst does not hit the rate limit.
                let mut results: Vec<Result<Vec<crate::hierarchy::ListRef>>> = Vec::new();
                for chunk in spaces.chunks(4) {
                    let part: Vec<Result<Vec<crate::hierarchy::ListRef>>> = thread::scope(|s| {
                        let jobs: Vec<_> = chunk
                            .iter()
                            .map(|(id, name)| s.spawn(|| cu.space_lists(id, name)))
                            .collect();
                        jobs.into_iter()
                            .map(|j| j.join().expect("space thread panicked"))
                            .collect()
                    });
                    results.extend(part);
                }
                // One unreadable space should not hide the others.
                let failed = results.iter().filter(|r| r.is_err()).count();
                let lists = results.into_iter().flatten().flatten().collect();
                Ok((spaces, lists, cu.my_user_id().ok(), failed))
            })();
            let _ = tx.send(Msg::Hierarchy(result));
        });
    }

    fn load_tasks(&mut self, source: Source, closed: bool) {
        let key = task_key(&source, closed);
        if !self.state.loading_tasks.insert(key.clone()) {
            return;
        }
        let (cu, tx, spaces) = (self.cu.clone(), self.tx.clone(), self.cfg.space_ids.clone());
        thread::spawn(move || {
            let result = match &source {
                Source::Mine => cu.my_tasks(&spaces, closed),
                Source::List(l) => cu.list_tasks(&l.id, closed),
            };
            let _ = tx.send(Msg::Tasks(key, result));
        });
    }

    fn receive(&mut self, msg: Msg) {
        if let Some(dir) = cache::state_dir() {
            match &msg {
                Msg::Hierarchy(Ok((spaces, lists, me, _))) => {
                    cache::write_hierarchy(
                        &dir,
                        &cache::CachedHierarchy {
                            spaces: spaces.clone(),
                            lists: lists.clone(),
                            me: *me,
                        },
                    );
                }
                Msg::Tasks(key, Ok(tasks)) => cache::write_tasks(&dir, key, tasks),
                _ => {}
            }
        }
        let refresh = self.state.receive(msg);
        if refresh {
            self.read_state();
        }
    }

    fn read_state(&mut self) {
        self.pending_read = Instant::now();
        self.state.set_links(store::read().panes);
    }

    fn apply(&mut self, effect: Effect) {
        match effect {
            Effect::LoadTasks { source, closed } => self.load_tasks(source, closed),
            Effect::Refresh => {
                self.state.status = REFRESHING.into();
                self.load_tasks(self.state.source.clone(), self.state.show_closed);
                self.load_hierarchy();
            }
            Effect::Start {
                task,
                mode,
                agent,
                repos,
            } => self.start(task, mode, &agent, repos),
            Effect::Implement { task, pane, change } => self.implement(task, &pane, &change),
            Effect::AddRepo { task, repo } => self.add_repo(task, repo),
            Effect::ChooseAgent { task, mode } => self.choose_agent(task, mode),
            Effect::ChooseRepo { task, mode, agent } => self.choose_repo(task, mode, agent),
            Effect::ChooseAddRepo(task) => self.choose_add_repo(task),
            Effect::PreviewClose(task) => self.preview_close(task),
            Effect::CloseTask(task) => self.close_task(task),
            Effect::PostUpdate { task, update } => self.post(task, update),
            Effect::DiscardUpdate(task) => self.discard(task),
            Effect::EditDraft { task, update } => self.edit_request = Some((task, update)),
            Effect::LoadDetail(task) => self.load_detail(task),
            Effect::ChooseTeam => self.choose_team(),
            Effect::SwitchTeam { id, name } => self.switch_team(id, name),
            Effect::OpenUrl(url) => {
                if let Err(e) = app::open_url(&url) {
                    self.state.status = format!("could not open the browser: {e:#}");
                }
            }
        }
    }

    fn load_detail(&mut self, task: Task) {
        let (cu, tx) = (self.cu.clone(), self.tx.clone());
        let id = task.id.clone();
        let list = task.list_id.clone();
        thread::spawn(move || {
            let ticket = cu.ticket(&id).map(Box::new);
            let statuses = match list {
                Some(list) => cu.list_statuses(&list),
                None => Ok(Vec::new()),
            };
            let _ = tx.send(Msg::Detail(id, ticket, statuses));
        });
    }

    fn choose_team(&mut self) {
        let (cu, tx) = (self.cu.clone(), self.tx.clone());
        thread::spawn(move || {
            let _ = tx.send(Msg::Teams(cu.teams()));
        });
    }

    fn switch_team(&mut self, id: String, name: String) {
        store::remember("team", &id);
        self.cu = ClickUp::new(self.cfg.token.clone(), Some(id));
        if let Some(dir) = cache::state_dir() {
            cache::clear(&dir);
        }
        self.state.switch_team();
        self.state.status = format!("workspace: {name}");
        self.load_hierarchy();
        self.load_tasks(Source::Mine, false);
    }

    fn choose_agent(&mut self, task: Task, mode: Mode) {
        let choices = app::agent_options(&self.cfg);
        if choices.is_empty() {
            self.state.status = NO_AGENT.into();
        } else {
            self.state.open_agent(task, mode, choices);
        }
    }

    fn choose_repo(&mut self, task: Task, mode: Mode, agent: String) {
        let choices = app::repo_options();
        if choices.is_empty() {
            self.state.status = NO_REPO.into();
        } else {
            self.state.open_repo(task, mode, agent, choices);
        }
    }

    fn choose_add_repo(&mut self, task: Task) {
        let Some((_, link)) = self.state.links.get(&task.id).cloned() else {
            self.state.status = "this task has no running worktree".into();
            return;
        };
        if link.folder().is_none() {
            self.state.status = "only a task started with 2+ repos has a task folder".into();
            return;
        }
        let mut choices = app::repo_options();
        choices.retain(|r| {
            !link
                .worktrees
                .iter()
                .any(|w| taskspace::same_repo(&w.repo, r))
        });
        if choices.is_empty() {
            self.state.status = "every open repo is already in this task".into();
        } else {
            self.state.open_add_repo(task, choices);
        }
    }

    fn preview_close(&mut self, task: Task) {
        match app::close_preview(&task.id) {
            Ok(preview) => self.state.open_close(task, preview),
            Err(e) => self.state.status = format!("cannot close {}: {e:#}", task.name),
        }
    }

    fn start(&mut self, task: Task, mode: Mode, agent: &str, repos: Vec<String>) {
        let what = if mode == Mode::Spec {
            "writing the spec for"
        } else {
            "starting"
        };
        self.state.status = format!("{agent}: {what} {}…", task.name);
        let (cfg, cu, tx) = (self.cfg.clone(), self.cu.clone(), self.tx.clone());
        let agent = agent.to_string();
        thread::spawn(move || {
            let progress_tx = tx.clone();
            let progress = move |m: String| drop(progress_tx.send(Msg::Progress(m)));
            let result = app::start(&cfg, &cu, &task, &agent, &repos, mode, &progress);
            let _ = tx.send(Msg::Started(task.id, result));
        });
    }

    fn implement(&mut self, task: Task, pane: &str, change: &str) {
        let (tx, pane, change) = (self.tx.clone(), pane.to_string(), change.to_string());
        let id = task.id;
        thread::spawn(move || {
            let _ = tx.send(Msg::Implemented(id, app::implement(&pane, &change)));
        });
    }

    fn add_repo(&mut self, task: Task, repo: String) {
        self.state.status = format!("adding {} to {}…", taskspace::repo_name(&repo), task.name);
        let (tx, id) = (self.tx.clone(), task.id);
        thread::spawn(move || drop(tx.send(Msg::Added(app::add_repo(&id, &repo)))));
    }

    fn close_task(&mut self, task: Task) {
        self.state.status = format!("closing {}…", task.name);
        let (tx, id) = (self.tx.clone(), task.id);
        thread::spawn(move || drop(tx.send(Msg::Closed(app::close_task(&id)))));
    }

    fn post(&mut self, task: Task, update: Update) {
        self.state.status = format!("posting to {}…", task.name);
        let (cu, tx) = (self.cu.clone(), self.tx.clone());
        thread::spawn(move || drop(tx.send(Msg::Posted(app::post_update(&cu, &task.id, &update)))));
    }

    fn discard(&mut self, task: Task) {
        self.state.status = match app::clear_pending(&task.id) {
            Ok(()) => format!("discarded the drafted update for {}", task.name),
            Err(e) => format!("could not discard: {e:#}"),
        };
        self.read_state();
    }

    fn draw(&mut self, f: &mut Frame) {
        let [main, footer] =
            Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).areas(f.area());
        let [left, right] =
            Layout::horizontal([Constraint::Length(36), Constraint::Min(20)]).areas(main);
        self.draw_side(f, left);
        self.draw_tasks(f, right);
        self.draw_footer(f, footer);
        match &self.state.modal {
            Modal::None => {}
            Modal::Detail => self.draw_detail(f),
            Modal::Agent {
                choices, sel, mode, ..
            } => {
                let title = if *mode == Mode::Spec {
                    " Write the spec with which agent? "
                } else {
                    " Code with which agent? "
                };
                draw_choices(f, title, choices, *sel)
            }
            Modal::Update { task, update } => draw_update(f, task, update),
            Modal::Who { choices, sel } => {
                let labels: Vec<String> = choices.iter().map(|(l, _)| l.clone()).collect();
                draw_choices(f, " Show tasks of ", &labels, *sel)
            }
            Modal::Repo {
                choices,
                sel,
                picked,
                mode,
                ..
            } => {
                let labels: Vec<String> = choices
                    .iter()
                    .zip(picked)
                    .map(|(r, p)| {
                        format!(
                            "[{}] {}",
                            if *p { "x" } else { " " },
                            taskspace::repo_name(r)
                        )
                    })
                    .collect();
                let title = if *mode == Mode::Spec {
                    " Repos for this spec · space marks · ⏎ starts · first marked holds the spec "
                } else {
                    " Repos · space marks · ⏎ starts · no marks starts the highlighted "
                };
                draw_choices(f, title, &labels, *sel)
            }
            Modal::AddRepo { choices, sel, .. } => {
                let names: Vec<String> = choices.iter().map(|r| taskspace::repo_name(r)).collect();
                draw_choices(f, " Add which repo? ", &names, *sel)
            }
            Modal::Close { task, preview } => draw_close(f, task, preview),
            Modal::Status { choices, sel, .. } => {
                draw_choices(f, " Change status to ", choices, *sel)
            }
            Modal::Team { choices, sel } => {
                let labels: Vec<String> = choices.iter().map(|(_, name)| name.clone()).collect();
                draw_choices(f, " ClickUp workspace ", &labels, *sel)
            }
        }
    }

    fn draw_side(&mut self, f: &mut Frame, area: Rect) {
        let rows = self.state.side_rows();
        let items: Vec<ListItem> = rows
            .iter()
            .map(|r| {
                let style = match r.kind {
                    RowKind::Header => Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    RowKind::Space(_) => Style::new().add_modifier(Modifier::BOLD),
                    _ => Style::new(),
                };
                ListItem::new(Span::styled(
                    format!("{}{}", "  ".repeat(r.depth), r.label),
                    style,
                ))
            })
            .collect();
        let title = if self.state.loading_hierarchy {
            " Sprints & Lists (loading…) "
        } else if self.state.stale.contains("hierarchy") {
            " Sprints & Lists (cached) "
        } else {
            " Sprints & Lists "
        };
        let list = List::new(items)
            .block(panel(title, self.state.focus == Focus::Side))
            .highlight_style(highlight());
        f.render_stateful_widget(list, area, &mut self.state.side);
    }

    fn draw_tasks(&mut self, f: &mut Frame, area: Rect) {
        let me = self.state.me;
        let tasks = self.state.visible_tasks();
        let rows: Vec<Row> = tasks
            .iter()
            .map(|t| {
                let who: Vec<&str> = t
                    .assignees
                    .iter()
                    .map(|a| first_name(&a.username))
                    .collect();
                let mine = me.is_some_and(|me| t.is_assigned_to(me));
                let closed = t.closed;
                let name = if self.state.pending.contains_key(&t.id) {
                    format!("✉ {}", t.name)
                } else {
                    t.name.clone()
                };
                Row::new([
                    Cell::from(t.status.clone()).style(if closed {
                        Style::new().dim()
                    } else {
                        Style::new().fg(Color::Cyan)
                    }),
                    Cell::from(
                        self.state
                            .phase_of(&t.id)
                            .map(Phase::label)
                            .unwrap_or_default(),
                    )
                    .style(if closed {
                        Style::new().dim()
                    } else {
                        Style::new().fg(Color::Yellow)
                    }),
                    Cell::from(name).style(if closed {
                        Style::new().dim()
                    } else {
                        Style::new()
                    }),
                    Cell::from(who.join(", ")).style(if mine {
                        Style::new().fg(Color::Green)
                    } else {
                        Style::new().dim()
                    }),
                ])
            })
            .collect();
        let count = tasks.len();
        let key = task_key(&self.state.source, self.state.show_closed);
        let loading = self.state.loading_tasks.contains(&key);
        let mut title = format!(" {} · {count}", self.state.source.title());
        if self.state.show_closed {
            title.push_str(" · closed");
        }
        if !self.state.who_label.is_empty() {
            title.push_str(&format!(" · {}", self.state.who_label));
        }
        if !self.state.filter.is_empty() {
            title.push_str(&format!(" · \"{}\"", self.state.filter));
        }
        if loading {
            title.push_str(" (loading…) ");
        } else if self.state.stale.contains(&key) {
            title.push_str(" (cached) ");
        } else {
            title.push(' ');
        }
        let widths = [
            Constraint::Length(16),
            Constraint::Length(7),
            Constraint::Min(20),
            Constraint::Length(18),
        ];
        let table = Table::new(rows, widths)
            .header(Row::new(["STATUS", "PHASE", "NAME", "WHO"]).style(Style::new().bold().dim()))
            .block(panel(&title, self.state.focus == Focus::Tasks))
            .row_highlight_style(highlight());
        if self.state.table.selected().is_none_or(|s| s >= count) {
            self.state.table.select((count > 0).then_some(0));
        }
        f.render_stateful_widget(table, area, &mut self.state.table);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let line = if self.state.filtering {
            Line::from(vec![
                Span::raw(" filter: ").bold(),
                Span::raw(&self.state.filter),
                Span::raw("▏"),
            ])
        } else if !self.state.status.is_empty() {
            Line::from(format!(" {}", self.state.status)).fg(Color::Yellow)
        } else {
            let mut help = " ↑↓ move  ⏎ open  s spec  c code  C closed  W workspace  A add repo  X close  u review ✉  o browser  / filter  m mine  a assignee  r refresh  q quit".to_string();
            if self.state.can_implement() {
                help.push_str("  i implement");
            }
            Line::from(help).dim()
        };
        f.render_widget(Paragraph::new(line), area);
    }

    fn draw_detail(&self, f: &mut Frame) {
        let Some(t) = self.state.selected_task() else {
            return;
        };
        let area = centered(f.area(), 80, 70);
        let who: Vec<&str> = t.assignees.iter().map(|a| a.username.as_str()).collect();
        let mut text = vec![
            Line::from(t.name.clone()).bold(),
            Line::from(vec![
                Span::raw(t.status.clone()).fg(Color::Cyan),
                Span::raw(format!("  ·  {}", who.join(", "))),
            ]),
            Line::from(t.url.clone()).dim(),
        ];
        if let Some((_, link)) = self.state.links.get(&t.id) {
            text.push(Line::from(format!("Branch: {}", link.branch)).dim());
            if !link.worktrees.is_empty() {
                let repos: Vec<String> = link
                    .worktrees
                    .iter()
                    .map(|w| taskspace::repo_name(&w.repo))
                    .collect();
                text.push(Line::from(format!("Repos: {}", repos.join(", "))).dim());
            }
            if let Some(folder) = link.folder() {
                text.push(Line::from(format!("Task folder: {folder}")).dim());
            }
        }
        text.extend([Line::from(""), Line::from(t.description.trim().to_string())]);
        text.push(Line::from(""));
        match self.state.tickets.get(&t.id) {
            Some(ticket) => {
                text.push(Line::from(format!("Comments ({})", ticket.comments.len())).bold());
                if ticket.comments.is_empty() {
                    text.push(Line::from("No comments.").dim());
                }
                for c in &ticket.comments {
                    text.push(
                        Line::from(format!("{} · {}", c.user, crate::ticket::date(c.date_ms)))
                            .dim(),
                    );
                    for line in c.text.lines() {
                        text.push(Line::from(line.to_string()));
                    }
                    text.push(Line::from(""));
                }
            }
            None => text.push(Line::from("Loading comments…").dim()),
        }
        f.render_widget(Clear, area);
        let title = if self.state.can_implement() {
            " Task · t status  s spec  i implement  c code  A add repo  X close  o browser  esc "
        } else {
            " Task · t status  s spec  c code  A add repo  X close  o browser  esc "
        };
        let block = panel(title, true);
        f.render_widget(
            Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
            area,
        );
    }
}

fn draw_choices(f: &mut Frame, title: &str, choices: &[String], sel: usize) {
    let height = choices.len() as u16 + 2;
    let width = choices
        .iter()
        .map(|c| c.chars().count())
        .chain([title.chars().count()])
        .max()
        .unwrap_or(0) as u16
        + 4;
    let full = f.area();
    let area = Rect {
        x: full.width.saturating_sub(width) / 2,
        y: full.height.saturating_sub(height) / 2,
        width: width.min(full.width),
        height: height.min(full.height),
    };
    let items: Vec<ListItem> = choices
        .iter()
        .map(|c| ListItem::new(format!(" {c}")))
        .collect();
    let list = List::new(items)
        .block(panel(title, true))
        .highlight_style(highlight());
    f.render_widget(Clear, area);
    f.render_stateful_widget(
        list,
        area,
        &mut ListState::default().with_selected(Some(sel)),
    );
}

fn draw_update(f: &mut Frame, task: &Task, update: &Update) {
    let area = centered(f.area(), 70, 50);
    let mut text = vec![
        Line::from(format!("Post this to ClickUp task \"{}\"?", task.name)).bold(),
        Line::from(""),
    ];
    if update.comment.trim().is_empty() {
        text.push(Line::from("Status only, no comment.").dim());
    } else {
        text.extend([
            Line::from("Comment:").dim(),
            Line::from(update.comment.clone()),
        ]);
    }
    if let Some(s) = &update.status {
        text.extend([
            Line::from(""),
            Line::from("Status:").dim(),
            Line::from(format!("→ {s}")),
        ]);
    }
    text.extend([
        Line::from(""),
        Line::from("Nothing is posted until you press Enter.").fg(Color::Yellow),
    ]);
    f.render_widget(Clear, area);
    let block = panel(
        " Drafted ClickUp update · e edit  ⏎ post  d discard  esc later ",
        true,
    );
    f.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_close(f: &mut Frame, task: &Task, preview: &app::ClosePreview) {
    let area = centered(f.area(), 70, 50);
    let mut text = vec![
        Line::from(format!("Close \"{}\"?", task.name)).bold(),
        Line::from(""),
        Line::from(format!("Branch: {}", preview.branch)).dim(),
    ];
    if !preview.worktrees.is_empty() {
        let repos: Vec<String> = preview
            .worktrees
            .iter()
            .map(|w| taskspace::repo_name(&w.repo))
            .collect();
        text.push(Line::from(format!("Removes: {}", repos.join(", "))).dim());
    }
    if let Some(folder) = &preview.folder {
        text.push(Line::from(format!("Removes folder: {folder}")).dim());
        text.push(Line::from("Files at the task-folder root are removed too.").dim());
    }
    if preview.dirty.is_empty() {
        text.push(Line::from("No uncommitted changes.").fg(Color::Green));
    } else {
        let names: Vec<String> = preview
            .dirty
            .iter()
            .map(|r| taskspace::repo_name(r))
            .collect();
        text.push(
            Line::from(format!(
                "Warning: uncommitted changes in {}.",
                names.join(", ")
            ))
            .fg(Color::Red),
        );
    }
    text.extend([
        Line::from(""),
        Line::from("Enter removes every worktree and closes the workspace. This cannot be undone.")
            .fg(Color::Yellow),
    ]);
    f.render_widget(Clear, area);
    let block = panel(" Close task · ⏎ close  esc cancel ", true);
    f.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn panel(title: &str, focused: bool) -> Block<'static> {
    let style = if focused {
        Style::new().fg(Color::Blue)
    } else {
        Style::new().dim()
    };
    Block::bordered()
        .title(title.to_string())
        .border_style(style)
}

fn highlight() -> Style {
    Style::new().add_modifier(Modifier::REVERSED)
}

fn centered(area: Rect, pct_w: u16, pct_h: u16) -> Rect {
    let w = area.width * pct_w / 100;
    let h = area.height * pct_h / 100;
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

fn first_name(username: &str) -> &str {
    username.split_whitespace().next().unwrap_or(username)
}
