//! Tier 2: Boundary & Corner Cases for contract structure and BLAKE3 baseline tamper protection.

mod common;

use common::{TestWorkspace, run_slugrails};

#[test]
fn test_missing_contract_exits_with_error() {
    let ws = TestWorkspace::new("contract_missing");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"no_contract\"\nedition = \"2024\"\n",
    );
    ws.write_file("src/lib.rs", "pub fn ok() {}\n");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    // Missing contract must exit with code 2 or error
    assert!(res.exit_code == 2 || res.exit_code == 1);
}

#[test]
fn test_unclosed_frontmatter_fences_fails_with_exit_code_2() {
    let ws = TestWorkspace::new("contract_unclosed");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"unclosed\"\nedition = \"2024\"\n",
    );
    // Opening +++ without closing +++
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\n# Missing closing fence\n",
    );
    ws.write_file("src/lib.rs", "pub fn ok() {}\n");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(
        res.exit_code, 2,
        "Unclosed frontmatter must return exit code 2 (ContractError)"
    );
}

#[test]
fn test_missing_frontmatter_fences_falls_back_gracefully() {
    let ws = TestWorkspace::new("contract_no_fences");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"no_fences\"\nedition = \"2024\"\n",
    );
    // Plain markdown document without +++ fences
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "# Pure Markdown Architecture\nNo frontmatter declared.\n",
    );
    ws.write_file("src/lib.rs", "pub fn ok() {}\n");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    // Should fall back gracefully to default empty contract (code 0) or incomplete (code 2)
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_tampered_contract_baseline_hash_fails_exit_code_2() {
    let ws = TestWorkspace::new("contract_tampered");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"tampered\"\nedition = \"2024\"\n",
    );
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Tampered Contract\n",
    );
    // Write invalid/mismatched BLAKE3 hash
    ws.write_file(
        ".planning/slugrails/contract.baseline",
        "0000000000000000000000000000000000000000000000000000000000000000\n",
    );
    ws.write_file("src/lib.rs", "pub fn ok() {}\n");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(res.exit_code, 2, "Hash mismatch must trigger exit code 2");
    let err = format!("{}{}", res.stdout, res.stderr);
    assert!(err.contains("baseline") || err.contains("hash") || err.contains("match"));
}

#[test]
fn test_missing_baseline_file_does_not_block_verification() {
    let ws = TestWorkspace::new("contract_fresh");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"fresh\"\nedition = \"2024\"\n",
    );
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Fresh Contract\n",
    );
    ws.write_file("src/lib.rs", "pub fn ok() {}\n");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert!(res.exit_code == 0 || res.exit_code == 2);
}

#[test]
fn test_corrupted_baseline_file_content_fails() {
    let ws = TestWorkspace::new("contract_corrupt_baseline");
    ws.write_file(
        "Cargo.toml",
        "[package]\nname = \"corrupt\"\nedition = \"2024\"\n",
    );
    ws.write_file(
        ".planning/ARCHITECTURE.md",
        "+++\nmodules = [\"core\"]\nlayers = [\"core\"]\nenforce = []\n+++\n# Contract\n",
    );
    // Write garbage content into baseline file
    ws.write_file(
        ".planning/slugrails/contract.baseline",
        "GARBAGE_NON_HEX_CONTENT_1234\n",
    );
    ws.write_file("src/lib.rs", "pub fn ok() {}\n");

    let res = run_slugrails(
        &["check", "--all", "--root", ws.root.to_str().unwrap()],
        &ws.root,
    );
    assert_eq!(
        res.exit_code, 2,
        "Corrupt baseline must trigger exit code 2"
    );
}
