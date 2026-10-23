use herdr_clickup::{
    cache::{self, CachedHierarchy},
    hierarchy::ListRef,
    task::{Assignee, Task},
};
use std::{fs, path::PathBuf};

fn temp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("herdr-clickup-cache-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn task() -> Task {
    Task {
        id: "86a".into(),
        name: "Fix login".into(),
        status: "to do".into(),
        url: "https://app.clickup.com/t/86a".into(),
        description: "After login we land on /404.".into(),
        assignees: vec![Assignee {
            id: 7,
            username: "Deepti".into(),
        }],
    }
}

fn hierarchy() -> CachedHierarchy {
    CachedHierarchy {
        spaces: vec![("s1".into(), "Space".into())],
        lists: vec![ListRef {
            id: "l1".into(),
            name: "Sprint".into(),
            space: "Space".into(),
            folder: Some("Weekly".into()),
            start: Some(1),
            due: Some(2),
            task_count: Some(3),
        }],
        me: Some(7),
    }
}

#[test]
fn the_hierarchy_survives_a_round_trip() {
    let dir = temp("hierarchy");
    cache::write_hierarchy(&dir, &hierarchy());
    assert_eq!(cache::read_hierarchy(&dir), Some(hierarchy()));
}

#[test]
fn tasks_survive_a_round_trip_and_the_key_is_safe() {
    let dir = temp("tasks");
    cache::write_tasks(&dir, "list/1", &[task()]);
    assert_eq!(cache::read_tasks(&dir, "list/1"), Some(vec![task()]));
    assert!(dir.join("cache/tasks-list_1.json").is_file());
}

#[test]
fn a_missing_or_corrupt_cache_is_ignored() {
    let dir = temp("corrupt");
    assert_eq!(cache::read_hierarchy(&dir), None);
    assert_eq!(cache::read_tasks(&dir, "mine"), None);
    fs::create_dir_all(dir.join("cache")).unwrap();
    fs::write(dir.join("cache/hierarchy.json"), "not json").unwrap();
    fs::write(dir.join("cache/tasks-mine.json"), "{").unwrap();
    assert_eq!(cache::read_hierarchy(&dir), None);
    assert_eq!(cache::read_tasks(&dir, "mine"), None);
}
