use crate::hook::State;
use anyhow::{Context, Result};
use std::{collections::HashSet, env, fs, path::PathBuf};

fn dir() -> Result<PathBuf> {
    env::var_os("HERDR_PLUGIN_STATE_DIR")
        .map(PathBuf::from)
        .context("HERDR_PLUGIN_STATE_DIR is not set; run through herdr")
}

/// Loads, changes and saves the state under an exclusive lock, so hooks running at once see each other's writes.
pub fn update<T>(change: impl FnOnce(&mut State) -> T) -> Result<T> {
    let dir = dir()?;
    let lock = fs::File::create(dir.join("panes.lock"))?;
    lock.lock()?;
    let path = dir.join("panes.json");
    let mut state: State = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let out = change(&mut state);
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(&state)?)?;
    fs::rename(tmp, path)?;
    Ok(out)
}

/// Removes links whose pane is not live. Returns how many links it dropped.
pub fn prune(live: &HashSet<String>) -> Result<usize> {
    update(|state| prune_state(state, live))
}

/// The pure part of `prune`, for tests.
pub fn prune_state(state: &mut State, live: &HashSet<String>) -> usize {
    let before = state.panes.len();
    state.panes.retain(|pane, _| live.contains(pane));
    before - state.panes.len()
}

/// The current state without taking the lock (writes are atomic renames), for display.
pub fn read() -> State {
    let Ok(dir) = dir() else {
        return State::default();
    };
    fs::read_to_string(dir.join("panes.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// A one-line choice remembered between runs (`agent`, `repo`), offered first next time.
pub fn last(name: &str) -> Option<String> {
    fs::read_to_string(dir().ok()?.join(name))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn remember(name: &str, value: &str) {
    if let Ok(d) = dir() {
        let _ = fs::write(d.join(name), value);
    }
}
