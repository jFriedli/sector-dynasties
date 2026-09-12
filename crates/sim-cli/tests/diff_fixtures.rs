//! Integration test for `sim-cli diff` (issue #96): proves the tool catches
//! a known, intentional difference between two SimState JSON snapshots
//! using a small fixture pair.
//!
//! `state_diff_after.json` is `state_diff_before.json` (a real tutorial-preset
//! `SimState`) with exactly one field changed: the dynasty founder's wealth.

use std::process::Command;

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn run_diff(extra_args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_sim-cli"))
        .arg("diff")
        .arg("--a")
        .arg(fixture("state_diff_before.json"))
        .arg("--b")
        .arg(fixture("state_diff_after.json"))
        .args(extra_args)
        .output()
        .expect("failed to run sim-cli diff");
    (
        output.status.success(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

#[test]
fn catches_the_intentional_wealth_difference_in_text_mode() {
    let (success, stdout, stderr) = run_diff(&[]);
    assert!(success, "sim-cli diff failed: {stderr}");
    assert!(stdout.contains("1 difference(s) found"));
    assert!(
        stdout.contains("dynasty.members[id=1].wealth: 2834.5154992733255 -> 7834.515499273326")
    );
}

#[test]
fn catches_the_intentional_wealth_difference_in_json_mode() {
    let (success, stdout, stderr) = run_diff(&["--json"]);
    assert!(success, "sim-cli diff --json failed: {stderr}");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output must be JSON");
    let entries = parsed.as_array().expect("top-level JSON must be an array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["path"], "dynasty.members[id=1].wealth");
    assert_eq!(entries[0]["before"], 2834.5154992733255);
    assert_eq!(entries[0]["after"], 7834.515499273326);
}

#[test]
fn reports_no_differences_between_a_fixture_and_itself() {
    let output = Command::new(env!("CARGO_BIN_EXE_sim-cli"))
        .arg("diff")
        .arg("--a")
        .arg(fixture("state_diff_before.json"))
        .arg("--b")
        .arg(fixture("state_diff_before.json"))
        .output()
        .expect("failed to run sim-cli diff");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "no differences found"
    );
}
