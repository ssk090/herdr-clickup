use herdr_clickup::config::token_from_command;

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
