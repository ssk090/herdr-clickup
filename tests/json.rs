use herdr_clickup::json::{arr, at, flag, num, text, u64_at};
use serde_json::json;

#[test]
fn text_and_arrays_are_empty_when_missing() {
    let v = json!({ "name": "Sprint", "items": [{ "id": 1 }], "none": null });
    assert_eq!(text(&v, "name"), "Sprint");
    assert_eq!(text(&v, "none"), "");
    assert_eq!(text(&v, "missing"), "");
    assert_eq!(arr(&v, "items").len(), 1);
    assert!(arr(&v, "missing").is_empty());
}

#[test]
fn numbers_accept_strings_and_numbers() {
    let v = json!({ "a": "42", "b": 7, "c": "x", "d": null });
    assert_eq!(num(&v, "a"), Some(42));
    assert_eq!(num(&v, "b"), Some(7));
    assert_eq!(num(&v, "c"), None);
    assert_eq!(num(&v, "d"), None);
    assert_eq!(num(&v, "missing"), None);
}

#[test]
fn pointers_read_strings_and_numbers() {
    let v = json!({ "task": { "id": "86a", "owner": 7 }, "flag": true });
    assert_eq!(at(&v, "/task/id").as_deref(), Some("86a"));
    assert_eq!(at(&v, "/task/missing"), None);
    assert_eq!(u64_at(&v, "/task/owner"), Some(7));
    assert_eq!(u64_at(&v, "/task/id"), None);
    assert_eq!(flag(&v, "flag"), Some(true));
    assert_eq!(flag(&v, "missing"), None);
}
