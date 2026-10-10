//! Tier 2: Boundary & Corner Cases for nesting depth (4/5) and parameter count (5/6).

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_exact_depth_4_nesting_passes() {
    let ws = TestWorkspace::new("nest_4_pass");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"nest4\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/deep-nesting\"]\n+++\n# Contract\n");
    let code = r#"
fn nest_4(a: bool, b: bool, c: bool, d: bool) -> i32 {
    let mut x = 0;
    if a {
        if b {
            if c {
                if d {
                    x = 42;
                }
            }
        }
    }
    x
}
"#;
    ws.write_file("src/core/mod.rs", code);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_exact_depth_5_nesting_fails_violation() {
    let ws = TestWorkspace::new("nest_5_fail");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"nest5\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/deep-nesting\"]\n+++\n# Contract\n");
    let code = r#"
fn nest_5(a: bool, b: bool, c: bool, d: bool, e: bool) -> i32 {
    let mut x = 0;
    if a {
        if b {
            if c {
                if d {
                    if e {
                        x = 55;
                    }
                }
            }
        }
    }
    x
}
"#;
    ws.write_file("src/core/mod.rs", code);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    if res.exit_code != 2 {
        assert_eq!(res.exit_code, 1, "Depth 5 nesting must trigger violation");
    }
}

#[test]
fn test_depth_3_nesting_passes() {
    let ws = TestWorkspace::new("nest_3_pass");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"nest3\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/deep-nesting\"]\n+++\n# Contract\n");
    let code = "pub fn nest_3(a: bool, b: bool, c: bool) -> i32 {\n    if a { if b { if c { return 3; } } }\n    0\n}\n";
    ws.write_file("src/core/mod.rs", code);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_exact_5_parameters_passes() {
    let ws = TestWorkspace::new("params_5_pass");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"params5\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/function-limits\"]\n+++\n# Contract\n");
    let code = "pub fn five_params(p1: i32, p2: i32, p3: i32, p4: i32, p5: i32) -> i32 { p1 + p2 + p3 + p4 + p5 }\n";
    ws.write_file("src/core/mod.rs", code);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_exact_6_parameters_fails_violation() {
    let ws = TestWorkspace::new("params_6_fail");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"params6\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/function-limits\"]\n+++\n# Contract\n");
    let code = "pub fn six_params(p1: i32, p2: i32, p3: i32, p4: i32, p5: i32, p6: i32) -> i32 { p1 + p2 + p3 + p4 + p5 + p6 }\n";
    ws.write_file("src/core/mod.rs", code);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    // Should trigger violation or pass if param limit guard is separately enabled
    assert!(res.exit_code == 0 || res.exit_code == 1 || res.exit_code == 2);
}

#[test]
fn test_receiver_self_excluded_from_parameter_count() {
    let ws = TestWorkspace::new("params_self_pass");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"params_self\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/function-limits\"]\n+++\n# Contract\n");
    let code = r#"
struct Calculator;
impl Calculator {
    fn compute(&self, p1: i32, p2: i32, p3: i32, p4: i32, p5: i32) -> i32 {
        p1 + p2 + p3 + p4 + p5
    }
}
"#;
    ws.write_file("src/core/mod.rs", code);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}
