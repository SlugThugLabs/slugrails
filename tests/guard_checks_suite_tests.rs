use slugrails::analyzer::test_region_lines;
use slugrails::contract::ArchitectureContract;
use slugrails::guards::checks::{
    DeepNestingCheck, NoSecretsCheck, SwallowedErrorsCheck, UnsafePolicyCheck,
};
use slugrails::guards::context::FileContext;
use slugrails::guards::traits::GuardCheck;
use slugrails::library::catalog::GuardCatalogEntry;
use slugrails::storage::ProjectExceptions;
use slugrails::types::Severity;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn build_catalog_entry(id: &str) -> GuardCatalogEntry {
    GuardCatalogEntry {
        id: id.to_string(),
        name: id.to_string(),
        category: "test".to_string(),
        summary: "test guard".to_string(),
        tags: vec![],
        aliases: vec![],
        file_path: PathBuf::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. hygiene/swallowed-errors tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_swallowed_errors_flags_discarded_results_and_empty_handlers() {
    let source = r#"
pub fn run_task() {
    let _ = std::fs::remove_file("temp.txt");

    match std::fs::read("config.json") {
        Ok(bytes) => println!("Read bytes: {}", bytes.len()),
        _ => {}
    }

    try {
        do_work();
    } catch {}
}
"#;

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("src/lib.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, source).unwrap();

    let guard = build_catalog_entry("hygiene/swallowed-errors");
    let exceptions = ProjectExceptions::default();
    let regions = test_region_lines(source);
    let rel = Path::new("src/lib.rs");

    let ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/lib.rs",
        content: source,
        is_test: false,
        test_regions: &regions,
        rule_name: "hygiene/swallowed-errors",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = SwallowedErrorsCheck;
    let contract = ArchitectureContract::default();
    let violations = check.check(&ctx, &contract);

    assert_eq!(
        violations.len(),
        3,
        "Expected 3 swallowed error violations (let _, _ => {{}}, and catch {{}}): {violations:?}"
    );

    // Verify all 3 violations are Error severity
    for v in &violations {
        assert_eq!(v.guard_id, "hygiene/swallowed-errors");
        assert_eq!(v.severity, Severity::Error);
    }

    // Check message contents
    assert!(violations.iter().any(|v| v.message.contains("let _ =")));
    assert!(violations.iter().any(|v| v.message.contains("_ => {}")));
    assert!(violations.iter().any(|v| v.message.contains("catch {}")));
}

#[test]
fn test_swallowed_errors_respects_test_regions_and_test_files() {
    let source = r#"
pub fn production_code() -> u32 {
    42
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_helper() {
        let _ = std::fs::remove_file("temp.txt");
        match Ok::<(), ()>(()) {
            _ => {}
        }
    }
}
"#;

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("src/lib.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, source).unwrap();

    let guard = build_catalog_entry("hygiene/swallowed-errors");
    let exceptions = ProjectExceptions::default();
    let regions = test_region_lines(source);
    let rel = Path::new("src/lib.rs");

    let ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/lib.rs",
        content: source,
        is_test: false,
        test_regions: &regions,
        rule_name: "hygiene/swallowed-errors",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = SwallowedErrorsCheck;
    let contract = ArchitectureContract::default();
    let violations = check.check(&ctx, &contract);

    assert!(
        violations.is_empty(),
        "Swallowed errors inside #[cfg(test)] must be ignored: {violations:?}"
    );

    // Also test test file convention (is_test: true)
    let test_file_ctx = FileContext {
        file: &file,
        rel_file: Path::new("tests/my_test.rs"),
        rel_str: "tests/my_test.rs",
        content: "fn test() { let _ = 1; }\n",
        is_test: true,
        test_regions: &[],
        rule_name: "hygiene/swallowed-errors",
        guard: &guard,
        exceptions: &exceptions,
    };
    let test_violations = check.check(&test_file_ctx, &contract);
    assert!(
        test_violations.is_empty(),
        "Swallowed errors in test file must be ignored: {test_violations:?}"
    );
}

#[test]
fn test_swallowed_errors_respects_valid_exceptions() {
    let dir = tempdir().expect("tempdir");
    let mut exceptions = ProjectExceptions::load(dir.path()).unwrap();
    let rel_file = Path::new("src/worker.rs");

    let entry = exceptions
        .add_exception(
            rel_file,
            "hygiene/swallowed-errors",
            "Fire-and-forget worker cleanup",
        )
        .unwrap();

    let source = format!(
        "// codeguard-exception: token={}; guard=hygiene/swallowed-errors;\n\
         pub fn cleanup() {{\n    let _ = std::fs::remove_file(\"/tmp/lock\");\n}}\n",
        entry.token
    );

    let abs_file = dir.path().join(rel_file);
    fs::create_dir_all(abs_file.parent().unwrap()).unwrap();
    fs::write(&abs_file, &source).unwrap();

    let guard = build_catalog_entry("hygiene/swallowed-errors");
    let regions = test_region_lines(&source);

    let ctx = FileContext {
        file: &abs_file,
        rel_file,
        rel_str: "src/worker.rs",
        content: &source,
        is_test: false,
        test_regions: &regions,
        rule_name: "hygiene/swallowed-errors",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = SwallowedErrorsCheck;
    let contract = ArchitectureContract::default();
    let violations = check.check(&ctx, &contract);

    assert!(
        violations.is_empty(),
        "Valid exception must suppress swallowed error violation: {violations:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. complexity/deep-nesting tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_deep_nesting_flags_excessive_nesting_and_passes_shallow() {
    let source = r#"
pub fn shallow_func() {
    if true {
        let x = 1;
    }
}

pub fn deeply_nested_func() {
    if true {
        while false {
            for _ in 0..1 {
                if true {
                    if true {
                        println!("Depth 5!");
                    }
                }
            }
        }
    }
}
"#;

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("src/algo.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, source).unwrap();

    let guard = build_catalog_entry("complexity/deep-nesting");
    let exceptions = ProjectExceptions::default();
    let regions = test_region_lines(source);
    let rel = Path::new("src/algo.rs");

    let ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/algo.rs",
        content: source,
        is_test: false,
        test_regions: &regions,
        rule_name: "complexity/deep-nesting",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = DeepNestingCheck;
    let contract = ArchitectureContract::default(); // default max_depth is 4
    let violations = check.check(&ctx, &contract);

    assert_eq!(
        violations.len(),
        1,
        "Only deeply_nested_func should be flagged: {violations:?}"
    );
    assert_eq!(violations[0].guard_id, "complexity/deep-nesting");
    assert_eq!(violations[0].severity, Severity::Error);
    assert!(violations[0].message.contains("deeply_nested_func"));
    assert!(violations[0].message.contains("exceeds maximum nesting depth"));
}

#[test]
fn test_deep_nesting_custom_contract_threshold() {
    let source = r#"
pub fn three_levels() {
    if true {
        if true {
            if true {
                println!("Level 3");
            }
        }
    }
}
"#;

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("src/algo.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, source).unwrap();

    let guard = build_catalog_entry("complexity/deep-nesting");
    let exceptions = ProjectExceptions::default();
    let regions = test_region_lines(source);
    let rel = Path::new("src/algo.rs");

    let ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/algo.rs",
        content: source,
        is_test: false,
        test_regions: &regions,
        rule_name: "complexity/deep-nesting",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = DeepNestingCheck;

    // With max_depth = 4 (default), level 3 passes
    let default_contract = ArchitectureContract::default();
    assert!(check.check(&ctx, &default_contract).is_empty());

    // With custom max_depth = 2, level 3 fails
    let mut guard_settings = BTreeMap::new();
    let mut deep_nesting_params = serde_json::Map::new();
    deep_nesting_params.insert("max_depth".to_string(), serde_json::json!(2));
    guard_settings.insert("deep_nesting".to_string(), serde_json::Value::Object(deep_nesting_params));

    let custom_contract = ArchitectureContract {
        guard_settings,
        ..Default::default()
    };

    let violations = check.check(&ctx, &custom_contract);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("three_levels"));
    assert!(violations[0].message.contains("(3/2)"));
}

#[test]
fn test_deep_nesting_ignores_test_regions_and_respects_exceptions() {
    let source = r#"
#[cfg(test)]
mod tests {
    fn deep_test() {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            assert!(true);
                        }
                    }
                }
            }
        }
    }
}
"#;

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("src/algo.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, source).unwrap();

    let guard = build_catalog_entry("complexity/deep-nesting");
    let exceptions = ProjectExceptions::default();
    let regions = test_region_lines(source);
    let rel = Path::new("src/algo.rs");

    let ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/algo.rs",
        content: source,
        is_test: false,
        test_regions: &regions,
        rule_name: "complexity/deep-nesting",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = DeepNestingCheck;
    let contract = ArchitectureContract::default();
    assert!(
        check.check(&ctx, &contract).is_empty(),
        "Test regions must be ignored for deep nesting"
    );

    // Test with approved exception
    let mut exceptions_mgr = ProjectExceptions::load(dir.path()).unwrap();
    let exc_entry = exceptions_mgr
        .add_exception(rel, "complexity/deep-nesting", "Legacy parsing machine")
        .unwrap();

    let exc_source = format!(
        "// codeguard-exception: token={}; guard=complexity/deep-nesting;\n\
         pub fn legacy_parse() {{\n\
             if true {{ if true {{ if true {{ if true {{ if true {{ }} }} }} }} }}\n\
         }}\n",
        exc_entry.token
    );
    let exc_ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/algo.rs",
        content: &exc_source,
        is_test: false,
        test_regions: &[],
        rule_name: "complexity/deep-nesting",
        guard: &guard,
        exceptions: &exceptions_mgr,
    };
    assert!(
        check.check(&exc_ctx, &contract).is_empty(),
        "Approved exception must suppress nesting violation"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. languages/rust/unsafe-policy tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unsafe_policy_flags_unsafe_blocks_and_functions_in_rs_files() {
    let source = r#"
pub unsafe fn raw_deref(ptr: *const u8) -> u8 {
    *ptr
}

pub fn safe_wrapper() {
    let x = 5;
    let ptr = &x as *const i32;
    unsafe {
        println!("Value: {}", *ptr);
    }
}
"#;

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("src/unsafe_mod.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, source).unwrap();

    let guard = build_catalog_entry("languages/rust/unsafe-policy");
    let exceptions = ProjectExceptions::default();
    let regions = test_region_lines(source);
    let rel = Path::new("src/unsafe_mod.rs");

    let ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/unsafe_mod.rs",
        content: source,
        is_test: false,
        test_regions: &regions,
        rule_name: "languages/rust/unsafe-policy",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = UnsafePolicyCheck;
    let contract = ArchitectureContract::default();
    let violations = check.check(&ctx, &contract);

    assert_eq!(
        violations.len(),
        2,
        "Expected 2 unsafe violations (unsafe fn and unsafe block): {violations:?}"
    );

    for v in &violations {
        assert_eq!(v.guard_id, "languages/rust/unsafe-policy");
        assert_eq!(v.severity, Severity::Error);
        assert!(v.message.contains("Unsafe code detected"));
    }
}

#[test]
fn test_unsafe_policy_ignores_non_rs_files_and_test_lines() {
    let py_source = "def unsafe_calculation():\n    return 'unsafe'\n";
    let dir = tempdir().expect("tempdir");
    let py_file = dir.path().join("src/script.py");
    fs::create_dir_all(py_file.parent().unwrap()).unwrap();
    fs::write(&py_file, py_source).unwrap();

    let guard = build_catalog_entry("languages/rust/unsafe-policy");
    let exceptions = ProjectExceptions::default();
    let rel_py = Path::new("src/script.py");

    let py_ctx = FileContext {
        file: &py_file,
        rel_file: rel_py,
        rel_str: "src/script.py",
        content: py_source,
        is_test: false,
        test_regions: &[],
        rule_name: "languages/rust/unsafe-policy",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = UnsafePolicyCheck;
    let contract = ArchitectureContract::default();
    assert!(
        check.check(&py_ctx, &contract).is_empty(),
        "Non-rs files must be completely ignored by unsafe-policy"
    );

    // Test inside #[cfg(test)]
    let rs_test_source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_unsafe() {
        unsafe {
            let _ = 1;
        }
    }
}
"#;
    let rs_file = dir.path().join("src/lib.rs");
    fs::write(&rs_file, rs_test_source).unwrap();
    let regions = test_region_lines(rs_test_source);
    let rs_ctx = FileContext {
        file: &rs_file,
        rel_file: Path::new("src/lib.rs"),
        rel_str: "src/lib.rs",
        content: rs_test_source,
        is_test: false,
        test_regions: &regions,
        rule_name: "languages/rust/unsafe-policy",
        guard: &guard,
        exceptions: &exceptions,
    };

    assert!(
        check.check(&rs_ctx, &contract).is_empty(),
        "Unsafe code inside #[cfg(test)] must not trigger violations"
    );
}

#[test]
fn test_unsafe_policy_respects_exceptions() {
    let dir = tempdir().expect("tempdir");
    let mut exceptions = ProjectExceptions::load(dir.path()).unwrap();
    let rel_file = Path::new("src/simd.rs");

    let entry = exceptions
        .add_exception(
            rel_file,
            "languages/rust/unsafe-policy",
            "Performance SIMD intrinsics",
        )
        .unwrap();

    let source = format!(
        "// codeguard-exception: token={}; guard=languages/rust/unsafe-policy;\n\
         pub fn fast_copy() {{\n    unsafe {{ std::ptr::null::<u8>(); }}\n}}\n",
        entry.token
    );

    let abs_file = dir.path().join(rel_file);
    fs::create_dir_all(abs_file.parent().unwrap()).unwrap();
    fs::write(&abs_file, &source).unwrap();

    let guard = build_catalog_entry("languages/rust/unsafe-policy");
    let ctx = FileContext {
        file: &abs_file,
        rel_file,
        rel_str: "src/simd.rs",
        content: &source,
        is_test: false,
        test_regions: &[],
        rule_name: "languages/rust/unsafe-policy",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = UnsafePolicyCheck;
    let contract = ArchitectureContract::default();
    assert!(
        check.check(&ctx, &contract).is_empty(),
        "Approved exception must suppress unsafe violation"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. hygiene/no-secrets tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_no_secrets_detects_and_masks_credentials() {
    let rsa_key = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0...\n-----END RSA PRIVATE KEY-----";
    let gh_token = format!("{}_{}", "ghp", "999999999999999999999999999999999999");
    let aws_key = format!("{}{}", "AKIA", "9999999999ABCDEF");
    let stripe_live = format!("{}_{}", "sk_live", "999999999999999999999999");

    let source = format!(
        "pub const RSA_KEY: &str = \"{rsa_key}\";\n\
         pub const GH_TOKEN: &str = \"{gh_token}\";\n\
         pub const AWS_KEY: &str = \"{aws_key}\";\n\
         pub const STRIPE_LIVE: &str = \"{stripe_live}\";\n"
    );

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("src/secrets.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, &source).unwrap();

    let guard = build_catalog_entry("hygiene/no-secrets");
    let exceptions = ProjectExceptions::default();
    let regions = test_region_lines(&source);
    let rel = Path::new("src/secrets.rs");

    let ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/secrets.rs",
        content: &source,
        is_test: false,
        test_regions: &regions,
        rule_name: "hygiene/no-secrets",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = NoSecretsCheck;
    let contract = ArchitectureContract::default();
    let violations = check.check(&ctx, &contract);

    assert_eq!(
        violations.len(),
        4,
        "Expected 4 secrets detected (RSA, GitHub, AWS, Stripe): {violations:?}"
    );

    for v in &violations {
        assert_eq!(v.guard_id, "hygiene/no-secrets");
        assert_eq!(v.severity, Severity::Error);
        assert!(v.message.contains("Potential hardcoded secret or token detected:"));
        assert!(v.message.contains("..."), "Secret must be masked with ellipsis: {}", v.message);

        // Security check: raw full secret tokens must NEVER appear in the violation message
        assert!(
            !v.message.contains(&gh_token),
            "GitHub token leaked in violation report!"
        );
        assert!(
            !v.message.contains(&aws_key),
            "AWS token leaked in violation report!"
        );
        assert!(
            !v.message.contains(&stripe_live),
            "Stripe token leaked in violation report!"
        );
    }

    // Verify masked prefixes
    assert!(violations.iter().any(|v| v.message.contains("'-----BEG...'")));
    assert!(violations.iter().any(|v| v.message.contains("'ghp_9999...'")));
    assert!(violations.iter().any(|v| v.message.contains("'AKIA9999...'")));
    assert!(violations.iter().any(|v| v.message.contains("'sk_live_...'")));
}

#[test]
fn test_no_secrets_ignores_test_lines_and_respects_exceptions() {
    let mock_aws = format!("{}{}", "AKIA", "8888888888ABCDEF");
    let mock_gh = format!("{}_{}", "ghp", "888888888888888888888888888888888888");

    let source = format!(
        "#[cfg(test)]\n\
         mod tests {{\n\
             const MOCK_AWS: &str = \"{mock_aws}\";\n\
             const MOCK_GH: &str = \"{mock_gh}\";\n\
         }}\n"
    );

    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("src/secrets.rs");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, &source).unwrap();

    let guard = build_catalog_entry("hygiene/no-secrets");
    let exceptions = ProjectExceptions::default();
    let regions = test_region_lines(&source);
    let rel = Path::new("src/secrets.rs");

    let ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/secrets.rs",
        content: &source,
        is_test: false,
        test_regions: &regions,
        rule_name: "hygiene/no-secrets",
        guard: &guard,
        exceptions: &exceptions,
    };

    let check = NoSecretsCheck;
    let contract = ArchitectureContract::default();
    assert!(
        check.check(&ctx, &contract).is_empty(),
        "Secrets inside test regions must be ignored"
    );

    // Test with approved exception
    let mut exceptions_mgr = ProjectExceptions::load(dir.path()).unwrap();
    let exc_entry = exceptions_mgr
        .add_exception(rel, "hygiene/no-secrets", "Sandbox test token")
        .unwrap();

    let exc_source = format!(
        "// codeguard-exception: token={}; guard=hygiene/no-secrets;\n\
         pub const KEY: &str = \"{mock_aws}\";\n",
        exc_entry.token
    );
    let exc_ctx = FileContext {
        file: &file,
        rel_file: rel,
        rel_str: "src/secrets.rs",
        content: &exc_source,
        is_test: false,
        test_regions: &[],
        rule_name: "hygiene/no-secrets",
        guard: &guard,
        exceptions: &exceptions_mgr,
    };
    assert!(
        check.check(&exc_ctx, &contract).is_empty(),
        "Approved exception must suppress secret violation"
    );
}

