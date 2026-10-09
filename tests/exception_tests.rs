#![allow(warnings, clippy::all, clippy::pedantic)]

#[cfg(test)]
mod tests {
    use slugrails::storage::ProjectExceptions;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_exception_lifecycle() {
        let dir = tempdir().unwrap();
        let project_path = dir.path();

        let mut exceptions = ProjectExceptions::load(project_path).unwrap();
        assert!(exceptions.exceptions.is_empty());

        let file = Path::new("src/server/transport.rs");
        let entry = exceptions
            .add_exception(file, "complexity/source-limits", "transport scaffolding")
            .unwrap();

        assert_eq!(entry.token.len(), 39, "canonical token is sr_tok_ + 32 hex");
        assert!(entry.token.starts_with("sr_tok_"));
        assert_eq!(exceptions.exceptions.len(), 1);

        // Verify lookup
        assert!(exceptions.is_exception_valid(file, "complexity/source-limits", &entry.token));
        // Wrong token fails
        assert!(!exceptions.is_exception_valid(file, "complexity/source-limits", "99999"));
        // Wrong guard fails
        assert!(!exceptions.is_exception_valid(file, "hygiene/no-secrets", &entry.token));
        // Wrong file fails
        assert!(!exceptions.is_exception_valid(
            Path::new("src/other.rs"),
            "complexity/source-limits",
            &entry.token
        ));

        // Revoke
        assert!(exceptions.revoke(&entry.token).unwrap());
        assert!(!exceptions.is_exception_valid(file, "complexity/source-limits", &entry.token));
    }

    #[test]
    fn test_exception_pure_in_memory() {
        let mut exceptions = ProjectExceptions::default();
        let file = Path::new("src/domain.rs");
        let entry =
            exceptions.insert_exception(file, "languages/rust/no-unwrap", "domain invariant");

        assert_eq!(entry.token.len(), 39, "canonical token is sr_tok_ + 32 hex");
        assert!(entry.token.starts_with("sr_tok_"));
        assert_eq!(exceptions.exceptions.len(), 1);
        assert!(exceptions.is_exception_valid(file, "languages/rust/no-unwrap", &entry.token));

        assert!(exceptions.remove_exception(&entry.token));
        assert!(!exceptions.is_exception_valid(file, "languages/rust/no-unwrap", &entry.token));
    }

    #[test]
    fn test_exception_custom_ttl() {
        let mut exceptions = ProjectExceptions::default();
        let file = Path::new("src/ttl.rs");
        let entry = exceptions.insert_exception(file, "hygiene/no-debug-prints", "temporary debug");

        // Valid under standard 30-day TTL and 60-day TTL
        assert!(exceptions.is_exception_valid_with_ttl(
            file,
            "hygiene/no-debug-prints",
            &entry.token,
            30
        ));
        assert!(exceptions.is_exception_valid_with_ttl(
            file,
            "hygiene/no-debug-prints",
            &entry.token,
            60
        ));
    }
}
