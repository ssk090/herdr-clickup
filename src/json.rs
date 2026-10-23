//! Helpers for the JSON that ClickUp and herdr send: numbers as strings, missing fields as null.
use serde_json::Value;

/// A string field, or an empty string when it is missing or not a string.
pub fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// An array field, or an empty slice when it is missing or not an array.
pub fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// A number field. ClickUp sends some numbers as strings and others as numbers.
pub fn num(v: &Value, key: &str) -> Option<i64> {
    match v.get(key)? {
        Value::String(s) => s.parse().ok(),
        n => n.as_i64(),
    }
}

/// A boolean field.
pub fn flag(v: &Value, key: &str) -> Option<bool> {
    v.get(key).and_then(Value::as_bool)
}

/// A string at a JSON pointer, such as `/status/status`.
pub fn at(v: &Value, pointer: &str) -> Option<String> {
    v.pointer(pointer)
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// A `u64` at a JSON pointer.
pub fn u64_at(v: &Value, pointer: &str) -> Option<u64> {
    v.pointer(pointer).and_then(Value::as_u64)
}
