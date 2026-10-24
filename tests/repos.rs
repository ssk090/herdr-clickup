use herdr_clickup::repos::repo_choices;

fn found() -> Vec<String> {
    ["/w/webui", "/w/api-hub", "/w/webui", "/w/other"]
        .map(String::from)
        .to_vec()
}

#[test]
fn found_repos_in_workspace_order_without_duplicates() {
    assert_eq!(
        repo_choices(&found(), None, None),
        ["/w/webui", "/w/api-hub", "/w/other"]
    );
}

#[test]
fn the_current_workspace_repo_comes_first_then_the_last_pick() {
    assert_eq!(
        repo_choices(&found(), Some("/w/other"), Some("/w/api-hub")),
        ["/w/other", "/w/api-hub", "/w/webui"]
    );
}

#[test]
fn the_last_pick_leads_when_the_current_workspace_is_not_a_repo() {
    assert_eq!(
        repo_choices(&found(), None, Some("/w/other")),
        ["/w/other", "/w/webui", "/w/api-hub"]
    );
}

#[test]
fn a_last_pick_no_longer_open_is_still_offered() {
    assert_eq!(repo_choices(&[], None, Some("/w/old")), ["/w/old"]);
}

#[test]
fn empty_paths_are_ignored() {
    assert!(repo_choices(&["".into()], Some(""), Some("")).is_empty());
}
