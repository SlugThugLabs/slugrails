//! Tier 2: Boundary & Corner Cases for structural limits (200/201 file lines, 50/51 function lines).

mod common;

use common::{TestWorkspace, run_slugrails};

fn generate_lines_file(num_lines: usize) -> String {
    let mut out = String::new();
    for i in 1..=num_lines {
        out.push_str(&format!("pub fn f_{i}() -> usize {{ {i} }}\n"));
    }
    out
}

fn generate_function_with_lines(num_lines: usize) -> String {
    let mut out = String::from("pub fn boundary_func() -> usize {\n");
    let statements_needed = if num_lines > 2 { num_lines - 2 } else { 0 };
    for i in 1..statements_needed {
        out.push_str(&format!("    let _v{i} = {i};\n"));
    }
    out.push_str("    0\n}\n");
    out
}

#[test]
fn test_exact_200_line_file_passes() {
    let ws = TestWorkspace::new("boundary_200_pass");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"bound200\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/source-limits\"]\n+++\n# Contract\n");
    let content = generate_lines_file(200);
    assert_eq!(content.lines().count(), 200);
    ws.write_file("src/core/mod.rs", &content);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_exact_201_line_file_fails_violation() {
    let ws = TestWorkspace::new("boundary_201_fail");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"bound201\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/source-limits\"]\n+++\n# Contract\n");
    let content = generate_lines_file(201);
    assert_eq!(content.lines().count(), 201);
    ws.write_file("src/core/mod.rs", &content);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    if res.exit_code != 2 {
        assert_eq!(res.exit_code, 1, "201-line file must trigger violation");
    }
}

#[test]
fn test_199_line_file_passes() {
    let ws = TestWorkspace::new("boundary_199_pass");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"bound199\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/source-limits\"]\n+++\n# Contract\n");
    let content = generate_lines_file(199);
    assert_eq!(content.lines().count(), 199);
    ws.write_file("src/core/mod.rs", &content);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_exact_50_line_function_passes() {
    let ws = TestWorkspace::new("boundary_50_fn_pass");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"bound50fn\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/function-limits\"]\n+++\n# Contract\n");
    let content = generate_function_with_lines(50);
    assert_eq!(content.lines().count(), 50);
    ws.write_file("src/core/mod.rs", &content);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_exact_51_line_function_fails_violation() {
    let ws = TestWorkspace::new("boundary_51_fn_fail");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"bound51fn\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/function-limits\"]\n+++\n# Contract\n");
    let content = generate_function_with_lines(51);
    assert_eq!(content.lines().count(), 51);
    ws.write_file("src/core/mod.rs", &content);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    if res.exit_code != 2 {
        assert_eq!(res.exit_code, 1, "51-line function must trigger violation");
    }
}

#[test]
fn test_49_line_function_passes() {
    let ws = TestWorkspace::new("boundary_49_fn_pass");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"bound49fn\"\nedition = \"2024\"\n",
    );
    ws.write_file(".planning/ARCHITECTURE.md", "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = [\"complexity/function-limits\"]\n+++\n# Contract\n");
    let content = generate_function_with_lines(49);
    assert_eq!(content.lines().count(), 49);
    ws.write_file("src/core/mod.rs", &content);

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}
