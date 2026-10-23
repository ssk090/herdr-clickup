use herdr_clickup::config::{pick, token_from_command};

#[test]
fn a_command_prints_the_token() {
    assert_eq!(token_from_command("printf 'pk_test'").unwrap(), "pk_test");
    assert_eq!(
        token_from_command("printf 'pk_test\\n\\n'").unwrap(),
        "pk_test"
    );
}

#[test]
fn a_failing_command_is_an_error() {
    assert!(token_from_command("printf 'no' >&2; exit 3").is_err());
}

#[test]
fn an_empty_command_is_an_error() {
    assert!(token_from_command("true").is_err());
}

#[test]
fn the_plugin_file_wins_over_the_environment() {
    assert_eq!(
        pick(Some("file".into()), Some("env".into())).as_deref(),
        Some("file")
    );
    assert_eq!(pick(None, Some("env".into())).as_deref(), Some("env"));
    assert_eq!(pick(Some("file".into()), None).as_deref(), Some("file"));
    assert_eq!(
        pick(Some(String::new()), Some("env".into())).as_deref(),
        Some("env")
    );
    assert_eq!(pick(None, None), None);
}
