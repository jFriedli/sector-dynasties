//! Structured diffing between two SimState (or any) JSON documents.
//!
//! Built for the `sim-cli diff` subcommand: a debugging aid so a developer
//! comparing two saved snapshots gets a short, legible list of what actually
//! changed instead of eyeballing two huge JSON blobs. See issue #96.
//!
//! Two things keep the output legible on real `SimState` saves:
//! - Object keys are compared by name, not position, so unrelated fields
//!   never shift the diff.
//! - Arrays whose elements are all objects carrying a scalar `id` field
//!   (systems, planets, countries, cities, dynasty members, ...) are matched
//!   by that `id` rather than by index, so inserting or removing one element
//!   doesn't produce a spurious diff for every element after it. Arrays of
//!   plain scalars (like an RNG state vector) fall back to index matching.

use serde_json::Value;

/// One leaf-level difference between two JSON documents.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffEntry {
    /// Dotted/bracketed path to the differing value, e.g.
    /// `sector.systems[id=7].planets[id=8].countries[id=9].government.franchise`.
    pub path: String,
    /// The value on the "before" side, or `None` if the path only exists on
    /// the "after" side (an addition).
    pub before: Option<Value>,
    /// The value on the "after" side, or `None` if the path only exists on
    /// the "before" side (a removal).
    pub after: Option<Value>,
}

impl DiffEntry {
    fn changed(path: String, before: Value, after: Value) -> Self {
        DiffEntry {
            path,
            before: Some(before),
            after: Some(after),
        }
    }

    fn added(path: String, after: Value) -> Self {
        DiffEntry {
            path,
            before: None,
            after: Some(after),
        }
    }

    fn removed(path: String, before: Value) -> Self {
        DiffEntry {
            path,
            before: Some(before),
            after: None,
        }
    }
}

/// Compares two JSON documents and returns every leaf-level difference,
/// ordered deterministically (by path) so output is stable across runs.
pub fn diff_json(a: &Value, b: &Value) -> Vec<DiffEntry> {
    let mut out = Vec::new();
    visit(String::new(), Some(a), Some(b), &mut out);
    out
}

fn visit(path: String, a: Option<&Value>, b: Option<&Value>, out: &mut Vec<DiffEntry>) {
    match (a, b) {
        (None, None) => {}
        (None, Some(bv)) => out.push(DiffEntry::added(path, bv.clone())),
        (Some(av), None) => out.push(DiffEntry::removed(path, av.clone())),
        (Some(av), Some(bv)) => {
            if av == bv {
                return;
            }
            match (av, bv) {
                (Value::Object(ao), Value::Object(bo)) => {
                    let mut keys: Vec<&String> = ao.keys().chain(bo.keys()).collect();
                    keys.sort();
                    keys.dedup();
                    for key in keys {
                        let child_path = if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{path}.{key}")
                        };
                        visit(child_path, ao.get(key), bo.get(key), out);
                    }
                }
                (Value::Array(aa), Value::Array(ba)) => diff_arrays(path, aa, ba, out),
                _ => out.push(DiffEntry::changed(path, av.clone(), bv.clone())),
            }
        }
    }
}

fn diff_arrays(path: String, a: &[Value], b: &[Value], out: &mut Vec<DiffEntry>) {
    if let (Some(a_by_id), Some(b_by_id)) = (index_by_id(a), index_by_id(b)) {
        let mut ids: Vec<&String> = a_by_id.keys().chain(b_by_id.keys()).collect();
        ids.sort();
        ids.dedup();
        for id in ids {
            let child_path = format!("{path}[id={id}]");
            visit(
                child_path,
                a_by_id.get(id).copied(),
                b_by_id.get(id).copied(),
                out,
            );
        }
        return;
    }

    let max_len = a.len().max(b.len());
    for i in 0..max_len {
        let child_path = format!("{path}[{i}]");
        visit(child_path, a.get(i), b.get(i), out);
    }
}

/// If every element of `arr` is an object with a scalar (string or number)
/// `id` field, returns a map from that id's JSON text to the element.
/// Returns `None` (fall back to index matching) for empty arrays, arrays of
/// scalars, or arrays with duplicate/missing/non-scalar ids.
fn index_by_id(arr: &[Value]) -> Option<std::collections::BTreeMap<String, &Value>> {
    if arr.is_empty() {
        return None;
    }
    let mut map = std::collections::BTreeMap::new();
    for element in arr {
        let id = element.as_object()?.get("id")?;
        if !(id.is_string() || id.is_number()) {
            return None;
        }
        let key = id.to_string();
        if map.insert(key, element).is_some() {
            return None; // duplicate id: not a reliable key, fall back
        }
    }
    Some(map)
}

/// Renders a JSON scalar for human-readable output: strings without their
/// surrounding quotes, everything else as compact JSON.
fn render(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Formats diff entries as one line per entry, sorted by path, for terminal
/// output. Returns a "no differences found" message when `entries` is empty.
pub fn format_text(entries: &[DiffEntry]) -> String {
    if entries.is_empty() {
        return "no differences found".to_string();
    }
    let mut lines = Vec::with_capacity(entries.len() + 1);
    lines.push(format!("{} difference(s) found:", entries.len()));
    for entry in entries {
        let line = match (&entry.before, &entry.after) {
            (Some(before), Some(after)) => {
                format!("  {}: {} -> {}", entry.path, render(before), render(after))
            }
            (None, Some(after)) => format!("  {}: + {}", entry.path, render(after)),
            (Some(before), None) => format!("  {}: - {}", entry.path, render(before)),
            (None, None) => continue,
        };
        lines.push(line);
    }
    lines.join("\n")
}

/// Formats diff entries as a JSON array of `{path, before, after}` objects
/// (missing sides omitted) for machine consumption.
pub fn format_json(entries: &[DiffEntry]) -> String {
    let values: Vec<Value> = entries
        .iter()
        .map(|entry| {
            let mut obj = serde_json::Map::new();
            obj.insert("path".to_string(), Value::String(entry.path.clone()));
            if let Some(before) = &entry.before {
                obj.insert("before".to_string(), before.clone());
            }
            if let Some(after) = &entry.after {
                obj.insert("after".to_string(), after.clone());
            }
            Value::Object(obj)
        })
        .collect();
    serde_json::to_string_pretty(&Value::Array(values)).expect("diff entries are valid JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identical_documents_have_no_differences() {
        let a = json!({"a": 1, "b": [1, 2, 3]});
        let b = a.clone();
        assert!(diff_json(&a, &b).is_empty());
    }

    #[test]
    fn catches_a_changed_scalar_field() {
        let a = json!({"dynasty": {"name": "House Meridian", "wealth": 100.0}});
        let b = json!({"dynasty": {"name": "House Meridian", "wealth": 250.5}});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "dynasty.wealth");
        assert_eq!(diffs[0].before, Some(json!(100.0)));
        assert_eq!(diffs[0].after, Some(json!(250.5)));
    }

    #[test]
    fn catches_an_added_and_a_removed_key() {
        let a = json!({"only_in_a": 1, "shared": 1});
        let b = json!({"only_in_b": 2, "shared": 1});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].path, "only_in_a");
        assert_eq!(diffs[0].before, Some(json!(1)));
        assert_eq!(diffs[0].after, None);
        assert_eq!(diffs[1].path, "only_in_b");
        assert_eq!(diffs[1].before, None);
        assert_eq!(diffs[1].after, Some(json!(2)));
    }

    #[test]
    fn matches_object_arrays_by_id_instead_of_index() {
        // Inserting a new city at the front must not report every existing
        // city as "changed" just because its index shifted.
        let a = json!({"cities": [
            {"id": 1, "population": 100},
            {"id": 2, "population": 200},
        ]});
        let b = json!({"cities": [
            {"id": 3, "population": 300},
            {"id": 1, "population": 100},
            {"id": 2, "population": 250},
        ]});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].path, "cities[id=2].population");
        assert_eq!(diffs[0].before, Some(json!(200)));
        assert_eq!(diffs[0].after, Some(json!(250)));
        assert_eq!(diffs[1].path, "cities[id=3]");
        assert_eq!(diffs[1].before, None);
    }

    #[test]
    fn scalar_arrays_fall_back_to_index_matching() {
        let a = json!({"state": [1, 2, 3]});
        let b = json!({"state": [1, 9, 3]});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "state[1]");
    }

    #[test]
    fn duplicate_ids_fall_back_to_index_matching_rather_than_panicking() {
        let a = json!([{"id": 1, "v": "x"}, {"id": 1, "v": "y"}]);
        let b = json!([{"id": 1, "v": "x"}, {"id": 1, "v": "z"}]);
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "[1].v");
    }

    #[test]
    fn format_text_reports_no_differences_found() {
        let diffs = diff_json(&json!({"a": 1}), &json!({"a": 1}));
        assert_eq!(format_text(&diffs), "no differences found");
    }

    #[test]
    fn format_text_is_legible_for_a_handful_of_changes() {
        let a = json!({"dynasty": {"wealth": 100.0}, "clock": {"tick": 360}});
        let b = json!({"dynasty": {"wealth": 250.5}, "clock": {"tick": 720}});
        let text = format_text(&diff_json(&a, &b));
        assert!(text.contains("2 difference(s) found:"));
        assert!(text.contains("clock.tick: 360 -> 720"));
        assert!(text.contains("dynasty.wealth: 100.0 -> 250.5"));
    }

    #[test]
    fn format_json_round_trips_through_serde_json() {
        let diffs = diff_json(&json!({"a": 1}), &json!({"a": 2}));
        let text = format_json(&diffs);
        let parsed: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed[0]["path"], json!("a"));
        assert_eq!(parsed[0]["before"], json!(1));
        assert_eq!(parsed[0]["after"], json!(2));
    }
}
