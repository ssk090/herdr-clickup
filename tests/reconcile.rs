use herdr_clickup::hook::{Link, State};
use herdr_clickup::ids::PaneId;
use herdr_clickup::store::prune_state;
use std::collections::HashSet;

fn link(task: &str) -> Link {
    Link {
        task_id: task.into(),
        branch: format!("CU-{task}"),
        change: None,
        phase: None,
        dir: None,
        worktrees: Vec::new(),
        title: String::new(),
        url: String::new(),
        agent: String::new(),
        working: false,
        reported: false,
        pending: None,
    }
}

#[test]
fn only_links_with_a_live_pane_stay() {
    let mut state = State::default();
    state.panes.insert("w1:p1".into(), link("1"));
    state.panes.insert("w1:p2".into(), link("2"));
    let live: HashSet<PaneId> = ["w1:p2"].map(PaneId::from).into();
    assert_eq!(prune_state(&mut state, &live), 1);
    assert!(state.panes.contains_key("w1:p2"));
    assert!(!state.panes.contains_key("w1:p1"));
}

#[test]
fn no_live_panes_drops_every_link() {
    let mut state = State::default();
    state.panes.insert("w1:p1".into(), link("1"));
    state.panes.insert("w1:p2".into(), link("2"));
    assert_eq!(prune_state(&mut state, &HashSet::new()), 2);
    assert!(state.panes.is_empty());
}
