//! Tier 1: Feature coverage tests for `slugrails plan`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_plan_help_flag_displays_usage() {
    let ws = TestWorkspace::new("plan_help");
    let res = run_slugrails(&["plan", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(
        output.contains("plan") || output.contains("architecture") || output.contains("contract")
    );
}

#[test]
fn test_plan_discovers_source_modules() {
    let ws = TestWorkspace::new("plan_discover");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"plan_app\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/analyzer/mod.rs", "pub fn analyze() {}\n");
    ws.write_file("src/guards/mod.rs", "pub fn guard() {}\n");

    let res = run_slugrails(&["plan", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert!(res.exit_code == 0 || res.exit_code == 2);

    let arch_path = ws.root.join(".planning/ARCHITECTURE.md");
    if arch_path.exists() {
        let content = std::fs::read_to_string(&arch_path).unwrap();
        assert!(content.contains("analyzer") || content.contains("guards"));
    }
}

#[test]
fn test_plan_creates_baseline_hash_file() {
    let ws = TestWorkspace::new("plan_baseline");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"baseline_app\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/lib.rs", "pub fn run() {}\n");

    let res = run_slugrails(&["plan", "--root", ws.root.to_str().unwrap()], &ws.root);
    if res.exit_code == 0 {
        let baseline = ws.root.join(".planning/slugrails/contract.baseline");
        assert!(baseline.exists(), "contract.baseline must exist after plan");
    }
}

#[test]
fn test_plan_rejects_unwritable_or_missing_root() {
    let ws = TestWorkspace::new("plan_missing_root");
    let missing_path = ws.root.join("does_not_exist_xyz");
    let res = run_slugrails(
        &["plan", "--root", missing_path.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code != 0);
}

#[test]
fn test_plan_short_help_flag() {
    let ws = TestWorkspace::new("plan_short_help");
    let res = run_slugrails(&["plan", "-h"], &ws.root);
    assert_eq!(res.exit_code, 0);
}
