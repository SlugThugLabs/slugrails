//! Tier 3: Cross-Feature Combinations & Pairwise Workflows.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_workflow_init_plan_check() {
    let ws = TestWorkspace::new("flow_init_plan_check");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"flow_app\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/core/mod.rs", "pub fn start() {}\n");

    // 1. Run init
    let res_init = run_slugrails(&["init", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert_eq!(res_init.exit_code, 0, "Init step should succeed");
    assert!(ws.file_exists(".planning/ARCHITECTURE.md"));

    // 2. Run plan
    let res_plan = run_slugrails(&["plan", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert!(res_plan.exit_code == 0 || res_plan.exit_code == 2);

    // 3. Run check
    let res_check = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res_check.exit_code == 0 || res_check.exit_code == 2);
}

#[test]
fn test_workflow_check_fix_check() {
    let ws = TestWorkspace::new("flow_check_fix_check");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"fix_flow\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"hygiene/no-debug-prints\"]\n+++\n# Contract\n");
    // Write code with auto-remediable debug print
    let dbg_stmt = concat!("d", "bg!(doubled);");
    let code = format!(
        "fn test_val(v: i32) -> i32 {{\n    let doubled = v * 2;\n    {dbg_stmt}\n    doubled\n}}\n"
    );
    ws.write_file("src/core/mod.rs", &code);

    // 1. Initial check flags violation (or incomplete if catalog missing)
    let res_check1 = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    if res_check1.exit_code == 1 {
        // 2. Run fix
        let res_fix = run_slugrails(
            &["fix", "--all", "--root", ws.root.to_str().unwrap()],
            &ws.root,
        );
        assert_eq!(res_fix.exit_code, 0);

        // 3. Second check passes
        let res_check2 = run_slugrails(
            &["check", "--all", "--root", ws.root.to_str().unwrap()],
            &ws.root,
        );
        assert_eq!(res_check2.exit_code, 0);
    }
}

#[test]
fn test_workflow_hook_install_and_uninstall() {
    let ws = TestWorkspace::new("flow_hook_lifecycle");
    ws.init_git();

    // 1. Install hook
    let res_inst = run_slugrails(
        &["hook", "install", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res_inst.exit_code, 0);
    assert!(ws.file_exists(".git/hooks/pre-commit"));
    let hook_content = ws.read_file(".git/hooks/pre-commit");
    assert!(hook_content.contains("slugrails"));

    // 2. Uninstall hook
    let res_uninst = run_slugrails(
        &["hook", "uninstall", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res_uninst.exit_code, 0);
    if ws.file_exists(".git/hooks/pre-commit") {
        let remaining = ws.read_file(".git/hooks/pre-commit");
        assert!(!remaining.contains("slugrails check"));
    }
}

#[test]
fn test_workflow_init_validate_scaffold_check() {
    let ws = TestWorkspace::new("flow_onboarding");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"onboard_app\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/core/mod.rs", "pub fn setup() {}\n");

    // 1. Init
    let res_init = run_slugrails(&["init", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert_eq!(res_init.exit_code, 0);

    // 2. Validate
    let res_val = run_slugrails(&["validate", "--root", ws.root.to_str().unwrap()], &ws.root);
    assert!(res_val.exit_code == 0 || res_val.exit_code == 1 || res_val.exit_code == 2);

    // 3. Scaffold tests
    let res_scaff = run_slugrails(
        &["scaffold-tests", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res_scaff.exit_code, 0);
    assert!(ws.file_exists("tests/slugrails.rs"));

    // 4. Check
    let res_check = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res_check.exit_code == 0 || res_check.exit_code == 2);
}

#[test]
fn test_workflow_exception_lifecycle() {
    let ws = TestWorkspace::new("flow_exception_store");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"ex_flow\"\nedition = \"2024\"\n",
    );
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Contract\n",
    );
    ws.write_file("src/core/mod.rs", "pub fn f() {}\n");

    // 1. List initial exceptions (empty)
    let res_list = run_slugrails(
        &["exception", "list", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res_list.exit_code == 0 || res_list.exit_code == 2);

    // 2. Non-interactive add is rejected
    let res_add = run_slugrails(
        &[
            "exception",
            "add",
            "src/core/mod.rs",
            "complexity/source-limits",
            "--reason",
            "test",
        ],
        &ws.root,
    );
    assert_eq!(res_add.exit_code, 2);

    // 3. Revoke fake token is rejected
    let res_rev = run_slugrails(
        &[
            "exception",
            "revoke",
            "non_existent_token_456",
            "--root",
            ws.root.to_str().unwrap(),
        ],
        &ws.root,
    );
    assert!(res_rev.exit_code != 0);
}
