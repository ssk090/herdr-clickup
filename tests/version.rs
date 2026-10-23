//! The plugin manifest and Cargo must name the same version, or `herdr plugin update` and the
//! release binary URL disagree.
use std::fs;

fn first_version(text: &str) -> Option<&str> {
    text.lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.split('"').next())
}

#[test]
fn cargo_and_the_plugin_manifest_share_a_version() {
    let cargo = fs::read_to_string("Cargo.toml").unwrap();
    let plugin = fs::read_to_string("herdr-plugin.toml").unwrap();
    let (a, b) = (
        first_version(&cargo).unwrap(),
        first_version(&plugin).unwrap(),
    );
    assert_eq!(
        a, b,
        "Cargo.toml and herdr-plugin.toml must have the same version"
    );
    assert!(!a.is_empty());
}
