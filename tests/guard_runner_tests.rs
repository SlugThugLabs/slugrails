#[cfg(test)]
mod tests {
    use slugrails::contract::ArchitectureContract;
    use slugrails::guards::run_guard_checks;
    use slugrails::library::builtins::get_builtin_guard_tests;
    use slugrails::library::catalog::GuardCatalog;
    use slugrails::storage::ProjectExceptions;
    use slugrails::types::GuardTestDefinition;
    use std::path::PathBuf;

    /// A `.unwrap()` inside an in-file `#[cfg(test)]` module is test code, while
    /// one in production code is still reported — both halves of the fix.
    #[test]
    fn in_file_test_module_is_not_reported_but_production_unwrap_is() {
        let source = "\
pub fn production() -> u8 {
    let x: Option<u8> = None;
    x.unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        let y: Option<u8> = None;
        y.unwrap();
    }
}
";
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).expect("create src dir");
        let file = root.join("src/lib.rs");
        std::fs::write(&file, source).expect("write source");

        let contract = ArchitectureContract {
            enforce: vec!["no_unwrap".to_string()],
            ..ArchitectureContract::default()
        };
        let definitions: Vec<(GuardTestDefinition, PathBuf)> = get_builtin_guard_tests()
            .into_iter()
            .map(|definition| (definition, PathBuf::from("/x")))
            .collect();
        let catalog = GuardCatalog::from_definitions(&definitions);
        let exceptions = ProjectExceptions::default();

        let report = run_guard_checks(root, &[file], &contract, &catalog, &exceptions)
            .expect("guard run succeeds");

        assert!(
            report.passed_tests.contains(&"no_unwrap".to_string()),
            "the rule must be evaluated, not skipped: {:?}",
            report.passed_tests
        );
        assert_eq!(
            report.violations.len(),
            1,
            "only the production unwrap may be reported: {:?}",
            report.violations
        );
        assert_eq!(
            report.violations[0].line,
            Some(3),
            "the production unwrap on line 3 is a true positive"
        );
    }
}
