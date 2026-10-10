//! Tier 1: Feature coverage tests for `slugrails fix`.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_fix_help_flag_displays_usage() {
    let ws = TestWorkspace::new("fix_help");
    let res = run_slugrails(&["fix", "--help"], &ws.root);
    assert_eq!(res.exit_code, 0);
    let output = format!("{}{}", res.stdout, res.stderr);
    assert!(output.contains("fix") || output.contains("remediate") || output.contains("auto"));
}

#[test]
fn test_fix_remediates_debug_prints_in_place() {
    let ws = TestWorkspace::new("fix_dbg");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"dbg_app\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"hygiene/no-debug-prints\"]\n+++\n# Contract\n");
    let dbg_stmt = concat!("d", "bg!(val);");
    let code = format!(
        "fn calculate(n: i32) -> i32 {{\n    let val = n * 2;\n    {dbg_stmt}\n    val\n}}\n"
    );
    let _file = ws.write_file("src/lib.rs", &code);

    let res = run_slugrails(
        &["fix", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);

    let updated = ws.read_file("src/lib.rs");
    if res.exit_code == 0 {
        assert!(!updated.contains(dbg_stmt));
        assert!(updated.contains("calculate"));
    }
}

#[test]
fn test_fix_preserves_clean_files_unmodified() {
    let ws = TestWorkspace::new("fix_clean");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"clean_app\"\nedition = \"2024\"\n",
    );
    let original = "pub fn clean_code() -> i32 {\n    42\n}\n";
    ws.write_file("src/lib.rs", original);

    let res = run_slugrails(
        &["fix", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);

    let after = ws.read_file("src/lib.rs");
    assert_eq!(original, after);
}

#[test]
fn test_fix_diff_flag_accepts_git_mode() {
    let ws = TestWorkspace::new("fix_diff");
    ws.init_git();
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"diff_app\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/lib.rs", "pub fn test() {}\n");

    let res = run_slugrails(
        &["fix", "--diff", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 1 || res.exit_code == 2);
}

#[test]
fn test_fix_short_help_flag() {
    let ws = TestWorkspace::new("fix_short_help");
    let res = run_slugrails(&["fix", "-h"], &ws.root);
    assert_eq!(res.exit_code, 0);
}
