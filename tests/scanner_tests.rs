#![allow(warnings, clippy::all, clippy::pedantic)]

#[cfg(test)]
mod tests {
    use slugrails::analyzer::{
        collect_source_files, count_code_lines, find_debug_prints, find_unwrap_expect_calls,
    };
    use slugrails::contract::ArchitectureContract;
    use slugrails::guards::run_guard_checks;
    use slugrails::library::catalog::GuardCatalog;
    use slugrails::storage::ProjectExceptions;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_tokenizer_code_line_count() {
        let code = r#"
// This is a comment
/* Multiline
   Comment */
fn main() {
    let x = 42; // inline comment
    let s = "string with // comment inside";
}
"#;
        assert_eq!(count_code_lines(code), 4);
    }

    #[test]
    fn test_tokenizer_unwrap_detection() {
        let code = r#"
fn test_unwraps() {
    // let c = opt.unwrap();
    let real = opt.unwrap();
    let s = "str.unwrap()";
}
"#;
        let unwraps = find_unwrap_expect_calls(code);
        assert_eq!(unwraps.len(), 1);
        assert_eq!(unwraps[0].0, 4); // Line 4 is the real unwrap
    }

    #[test]
    fn test_tokenizer_debug_print_detection() {
        let code = r#"
fn run() {
    // dbg!("ignored");
    console.log("real debug print");
    dbg!(x);
}
"#;
        let prints = find_debug_prints(code);
        assert_eq!(prints.len(), 2);
    }

    #[test]
    fn test_guard_runner_full_pass_and_fail() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let clean_file = src_dir.join("clean.rs");
        fs::write(&clean_file, "pub fn clean() -> Result<(), ()> { Ok(()) }\n").unwrap();

        let dirty_file = src_dir.join("dirty.rs");
        fs::write(
            &dirty_file,
            "pub fn dirty() { let x = Some(1).unwrap(); }\n",
        )
        .unwrap();

        let contract = ArchitectureContract {
            enforce: vec!["no_unwrap".to_string(), "source_limits".to_string()],
            ..Default::default()
        };

        let builtins = slugrails::library::builtins::get_builtin_guard_tests();
        let catalog = GuardCatalog::from_definitions(
            &builtins
                .into_iter()
                .map(|d| (d, std::path::PathBuf::new()))
                .collect::<Vec<_>>(),
        );
        let exceptions = ProjectExceptions::default();

        let report = run_guard_checks(
            dir.path(),
            &[clean_file, dirty_file],
            &contract,
            &catalog,
            &exceptions,
        )
        .unwrap();

        assert!(!report.is_pass());
        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].guard_id, "languages/rust/no-unwrap");
    }

    #[test]
    fn test_walker_skips_files_exceeding_size_ceiling() {
        let dir = tempdir().unwrap();
        let normal_file = dir.path().join("normal.rs");
        fs::write(&normal_file, "pub fn ok() {}\n").unwrap();

        let huge_file = dir.path().join("huge.rs");
        let huge_data = vec![b' '; 2 * 1024 * 1024 + 1024];
        fs::write(&huge_file, huge_data).unwrap();

        let collected = collect_source_files(dir.path()).unwrap();
        assert!(collected.contains(&normal_file));
        assert!(!collected.contains(&huge_file));
    }

    #[test]
    fn test_tokenizer_raw_strings_and_lifetimes() {
        let code = r##"
fn lifetime<'a, 'b>(x: &'a str) -> &'b str {
    let raw1 = r#"hello "world" unwrap() inside raw"#;
    let raw2 = r"plain unwrap() inside raw";
    let ch = 'z';
    x
}
"##;
        let unwraps = find_unwrap_expect_calls(code);
        assert_eq!(unwraps.len(), 0);
    }

    #[test]
    fn test_manifest_scanner_cargo_and_package_json() {
        let dir = tempdir().unwrap();
        let cargo_toml = dir.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-pkg"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.0"
"#,
        )
        .unwrap();

        let cargo_deps = slugrails::analyzer::manifest::scan_cargo_manifest(dir.path()).unwrap();
        assert!(cargo_deps.contains("serde"));
        assert!(cargo_deps.contains("tokio"));
        assert!(cargo_deps.contains("tempfile"));
        assert!(!cargo_deps.contains("actix"));

        let package_json = dir.path().join("package.json");
        fs::write(
            &package_json,
            r#"{
  "dependencies": {
    "react": "^18.0.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  }
}"#,
        )
        .unwrap();

        let js_deps = slugrails::analyzer::manifest::scan_package_json(dir.path()).unwrap();
        assert!(js_deps.contains("react"));
        assert!(js_deps.contains("typescript"));
        assert!(!js_deps.contains("vue"));
    }
}
