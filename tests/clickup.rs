use herdr_clickup::clickup::{parse_statuses, rate_limit_wait};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[test]
fn the_reset_header_sets_the_wait_and_is_capped() {
    let soon = (now() + 7).to_string();
    assert_eq!(rate_limit_wait(Some(&soon), None, 0), 7);
    let far = (now() + 3600).to_string();
    assert_eq!(rate_limit_wait(Some(&far), None, 0), 60);
}

#[test]
fn retry_after_and_backoff_fill_in_when_reset_is_absent() {
    assert_eq!(rate_limit_wait(None, Some("2"), 0), 2);
    assert_eq!(rate_limit_wait(Some("0"), Some("90"), 0), 60);
    assert_eq!(rate_limit_wait(None, None, 0), 1);
    assert_eq!(rate_limit_wait(None, None, 3), 8);
    assert_eq!(rate_limit_wait(None, None, 20), 60);
}

#[test]
fn list_statuses_keep_their_order() {
    let v = json!({ "statuses": [
        { "status": "to do", "type": "open" },
        { "status": "in progress", "type": "custom" },
        { "no": "status" }
    ]});
    assert_eq!(parse_statuses(&v), ["to do", "in progress"]);
    assert!(parse_statuses(&json!({})).is_empty());
}
