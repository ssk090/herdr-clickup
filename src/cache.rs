//! The last board data, so the board opens with content before ClickUp answers.
use crate::{hierarchy::ListRef, task::Task};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedHierarchy {
    pub spaces: Vec<(String, String)>,
    pub lists: Vec<ListRef>,
    pub me: Option<u64>,
}

/// `$HERDR_PLUGIN_STATE_DIR`, where the cache lives beside `panes.json`.
pub fn state_dir() -> Option<PathBuf> {
    env::var_os("HERDR_PLUGIN_STATE_DIR").map(PathBuf::from)
}

fn cache_dir(dir: &Path) -> PathBuf {
    dir.join("cache")
}

pub fn read_hierarchy(dir: &Path) -> Option<CachedHierarchy> {
    let text = fs::read_to_string(cache_dir(dir).join("hierarchy.json")).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn write_hierarchy(dir: &Path, data: &CachedHierarchy) {
    write(&cache_dir(dir).join("hierarchy.json"), data);
}

pub fn read_tasks(dir: &Path, key: &str) -> Option<Vec<Task>> {
    let text = fs::read_to_string(cache_dir(dir).join(file_name(key))).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn write_tasks(dir: &Path, key: &str, tasks: &[Task]) {
    write(&cache_dir(dir).join(file_name(key)), tasks);
}

fn file_name(key: &str) -> String {
    let safe: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("tasks-{safe}.json")
}

/// Best effort: a read-only or full state dir must not break the board.
fn write<T: Serialize + ?Sized>(path: &Path, data: &T) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let Ok(text) = serde_json::to_vec(data) else {
        return;
    };
    let tmp = path.with_extension("json.tmp");
    if fs::write(&tmp, text).is_ok() {
        let _ = fs::rename(tmp, path);
    }
}
