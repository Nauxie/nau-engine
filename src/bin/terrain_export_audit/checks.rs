use serde_json::{Value, json};
use std::path::PathBuf;

pub(crate) fn value_u64(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

pub(crate) fn value_f64(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

pub(crate) fn relative_path(value: &Value, key: &str) -> Option<PathBuf> {
    value.get(key).and_then(Value::as_str).map(PathBuf::from)
}

pub(crate) fn error_artifact(island: &str, kind: &str, error: &str) -> Value {
    json!({
        "island": island,
        "kind": kind,
        "error": error,
    })
}

pub(crate) fn check_at_least_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

pub(crate) fn check_at_least_f64(name: &str, value: f64, threshold: f64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

pub(crate) fn check_eq_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

pub(crate) fn check_eq_str(name: &str, value: &str, threshold: &str, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}
