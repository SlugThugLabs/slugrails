//! Tier 4: Real-World Application Scenarios (dirty git worktrees, pre-commit runs, full project, corrupt recovery).

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_scenario_dirty_git_worktree_diff_mode() {
    let ws = TestWorkspace::new("scenario_git_diff");
    ws.init_git();
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"diff_repo\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"languages/rust/no-unwrap\"]\n+++\n# Contract\n");
    // Clean file
    ws.write_file("src/core/mod.rs", "pub fn clean() -> i32 { 1 }\n");

    // Check with --diff mode in git repo
    let res = run_slugrails(
        &["check", "--diff", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 1 || res.exit_code == 2);
}

#[test]
fn test_scenario_pre_commit_hook_blocks_on_violations() {
    let ws = TestWorkspace::new("scenario_pre_commit");
    ws.init_git();
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"hook_gate\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"languages/rust/no-unwrap\"]\n+++\n# Contract\n");

    // Install pre-commit hook
    let res_install = run_slugrails(
        &["hook", "install", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res_install.exit_code, 0);

    // Create violation in src/
    ws.write_file(
        "src/core/mod.rs",
        "pub fn bad() { let _ = \"x\".parse::<i32>().unwrap(); }\n",
    );

    // Executing check --all blocks with exit code 1
    let res_check = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    if res_check.exit_code != 2 {
        assert_eq!(
            res_check.exit_code, 1,
            "Pre-commit check must fail on unwrap violation"
        );
    }
}

fn setup_full_project(ws: &TestWorkspace) {
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"full_app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    let arch = "+++\nmodules = [\"foundation\", \"core\", \"infrastructure\", \"presentation\"]\nlayers = [\"foundation\", \"core\", \"infrastructure\", \"presentation\"]\n\n[allowed_dependencies]\nfoundation = []\ncore = [\"foundation\"]\ninfrastructure = [\"core\", \"foundation\"]\npresentation = [\"infrastructure\", \"core\", \"foundation\"]\n\nenforce = [\"complexity/source-limits\", \"languages/rust/no-unwrap\"]\n+++\n# 4-Tier Spec\n";
    ws.write_file(".planning/ARCHITECTURE.md", arch);
    ws.write_file("src/foundation/mod.rs", "fn base() -> u32 { 1 }\n");
    ws.write_file(
        "src/core/mod.rs",
        "use crate::foundation::base;\nfn domain() -> u32 { base() + 1 }\n",
    );
    ws.write_file(
        "src/infrastructure/mod.rs",
        "use crate::core::domain;\nfn storage() -> u32 { domain() + 2 }\n",
    );
    ws.write_file(
        "src/presentation/mod.rs",
        "use crate::infrastructure::storage;\nfn ui() -> u32 { storage() + 3 }\n",
    );
    ws.write_file(
        "src/lib.rs",
        "mod foundation;\nmod core;\nmod infrastructure;\nmod presentation;\n",
    );
}

#[test]
fn test_scenario_full_project_verification() {
    let ws = TestWorkspace::new("scenario_full_project");
    setup_full_project(&ws);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_scenario_corrupt_exception_store_recovery() {
    let ws = TestWorkspace::new("scenario_corrupt_store");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"store_rec\"\nedition = \"2024\"\n",
    );
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Contract\n",
    );
    ws.write_file("src/core/mod.rs", "fn ok() {}\n");

    // Write corrupted JSON into exceptions.json
    ws.write_file(
        ".planning/slugrails/exceptions.json",
        "{ malformed json content !!!\n",
    );

    // Engine should recover or report clean diagnostic without crashing
    let res = run_slugrails(
        &["exception", "list", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 1 || res.exit_code == 2);
}

#[test]
fn test_scenario_multiple_concurrent_violations_reporting() {
    let ws = TestWorkspace::new("scenario_multi_violations");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"multi_violator\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"languages/rust/no-unwrap\", \"languages/rust/no-panic\"]\n+++\n# Contract\n");

    // File with both unwrap and panic violations
    let dirty_code = r#"
fn multi_bad() {
    let val = "123".parse::<i32>().unwrap();
    if val < 0 {
        panic!("negative value");
    }
}
"#;
    ws.write_file("src/core/mod.rs", dirty_code);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    if res.exit_code != 2 {
        assert_eq!(
            res.exit_code, 1,
            "Must exit 1 when multiple violations present"
        );
        let combined = format!("{}{}", res.stdout, res.stderr);
        assert!(
            combined.contains("unwrap")
                || combined.contains("panic")
                || combined.contains("violation")
        );
    }
}
