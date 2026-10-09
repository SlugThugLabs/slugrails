#![allow(warnings, clippy::all, clippy::pedantic)]

//! Exception-token contract tests.
//!
//! Canonical token: `sr_tok_` + 32 lowercase hex characters (128-bit keyed-BLAKE3 MAC
//! over path, guard, reason and grant date). Legacy 5-digit tokens are rejected.
//! See `.planning/adr/0002-exception-token-contract.md`.

use slugrails::storage::ProjectExceptions;
use slugrails::util::crypto::{compute_exception_token, compute_exception_token_with_date};
use std::collections::HashSet;
use std::path::Path;
use tempfile::tempdir;

/// A temp-project-backed store, so tests never write to the process CWD.
fn store() -> (tempfile::TempDir, ProjectExceptions) {
    let dir = tempdir().expect("tempdir");
    let exceptions = ProjectExceptions::load(dir.path()).expect("load empty store");
    (dir, exceptions)
}

fn assert_canonical(token: &str) {
    assert_eq!(token.len(), 39, "token must be sr_tok_ + 32 hex, got {token:?}");
    let hex = token
        .strip_prefix("sr_tok_")
        .unwrap_or_else(|| panic!("token must start with sr_tok_, got {token:?}"));
    assert_eq!(hex.len(), 32, "hex body must be 32 chars, got {hex:?}");
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "hex body must be hexadecimal, got {hex:?}"
    );
}

#[test]
fn test_token_is_canonical_sr_tok_format() {
    let token = compute_exception_token(Path::new("src/a.rs"), "complexity/source-limits", "why");
    assert_canonical(&token);
}

#[test]
fn test_token_is_deterministic_and_date_bound() {
    let file = Path::new("src/server.rs");
    let guard = "complexity/source-limits";
    let reason = "transport scaffolding";

    let a = compute_exception_token(file, guard, reason);
    let b = compute_exception_token(file, guard, reason);
    assert_eq!(a, b, "identical inputs must yield an identical token");
    assert_canonical(&a);

    let dated = compute_exception_token_with_date(file, guard, reason, "2026-01-01");
    assert_canonical(&dated);
    let dated2 = compute_exception_token_with_date(file, guard, reason, "2026-01-02");
    assert_ne!(dated, dated2, "the grant date is part of the MAC input");

    // The stored token must be reproducible from the entry's own fields.
    let (dir, mut exceptions) = store();
    let entry = exceptions
        .add_exception(file, guard, reason)
        .expect("add_exception");
    let grant_date = &entry.granted_at[..10];
    assert_eq!(
        entry.token,
        compute_exception_token_with_date(file, guard, reason, grant_date),
        "stored token must match the canonical derivation over its granted_at date"
    );
    drop(dir);
}

#[test]
fn test_token_separation_by_file_guard_and_reason() {
    let mut exceptions = ProjectExceptions::default();
    let file = Path::new("src/target.rs");
    let guard = "hygiene/no-debug-prints";

    let entry = exceptions.insert_exception(file, guard, "reported reason");
    assert_canonical(&entry.token);
    assert!(exceptions.is_exception_valid(file, guard, &entry.token));

    assert!(
        !exceptions.is_exception_valid(Path::new("src/other.rs"), guard, &entry.token),
        "token must not validate against a different file"
    );
    assert!(
        !exceptions.is_exception_valid(file, "hygiene/no-secrets", &entry.token),
        "token must not validate against a different guard"
    );
    let other_reason = exceptions.insert_exception(file, guard, "a different reason");
    assert_ne!(
        entry.token, other_reason.token,
        "a different reason must yield a different token"
    );
}

#[test]
fn test_store_dedupes_same_target_and_replaces_reason() {
    let (dir, mut exceptions) = store();
    let file = Path::new("src/worker.rs");
    let guard = "hygiene/swallowed-errors";

    let first = exceptions
        .add_exception(file, guard, "reason one")
        .expect("first add");
    assert_eq!(exceptions.exceptions.len(), 1);
    assert_canonical(&first.token);

    // Re-adding the same (file, guard) replaces the entry (and thus the reason/token).
    let second = exceptions
        .add_exception(file, guard, "reason one")
        .expect("idempotent add");
    assert_eq!(first.token, second.token, "same inputs must be idempotent");
    assert_eq!(exceptions.exceptions.len(), 1, "no duplicate entries");

    let replaced = exceptions
        .add_exception(file, guard, "reason two")
        .expect("replace add");
    assert_ne!(first.token, replaced.token, "new reason changes the token");
    assert_eq!(exceptions.exceptions.len(), 1, "still exactly one entry per target");
    assert!(exceptions.is_exception_valid(file, guard, &replaced.token));
    assert!(
        !exceptions.is_exception_valid(file, guard, &first.token),
        "the superseded token must no longer validate"
    );

    // Revoke removes it entirely.
    assert!(exceptions.remove_exception(&replaced.token));
    assert!(exceptions.exceptions.is_empty());
    drop(dir);
}

#[test]
fn test_legacy_five_digit_tokens_are_rejected() {
    let mut exceptions = ProjectExceptions::default();
    let file = Path::new("src/legacy.rs");
    let guard = "complexity/source-limits";

    // A syntactically plausible legacy token, stored as if it had been granted earlier.
    let legacy = slugrails::types::ExceptionEntry {
        token: "21483".to_string(),
        file: file.to_path_buf(),
        guard_id: guard.to_string(),
        reason: "legacy exemption".to_string(),
        granted_at: chrono::Utc::now().to_rfc3339(),
    };
    exceptions.exceptions.push(legacy);

    assert!(
        !exceptions.is_exception_valid(file, guard, "21483"),
        "legacy 5-digit tokens must not validate under the canonical contract"
    );
}

#[test]
fn test_malformed_and_forged_tokens_are_rejected() {
    let mut exceptions = ProjectExceptions::default();
    let file = Path::new("src/critical.rs");
    let guard = "complexity/source-limits";
    let valid = exceptions.insert_exception(file, guard, "audit reason").token;

    for candidate in [
        "",
        "sr_tok_",
        "sr_tok_0000",
        "sr_tok_00000000000000000000000000000000000000", // 38 hex chars
        "sr_tok_00000000000000000000000000000000",        // 32 zeros (well-formed but wrong MAC)
        "srtok_7662b37b047ccf9d29df0387f8a6eb37",         // wrong prefix
        "99999",
        "sr_tok_ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ", // non-hex body
    ] {
        assert!(
            !exceptions.is_exception_valid(file, guard, candidate),
            "malformed/forged token {candidate:?} must be rejected"
        );
    }

    // Truncating or mutating a valid token must invalidate it.
    let truncated = &valid[..valid.len() - 1];
    assert!(!exceptions.is_exception_valid(file, guard, truncated));
    let mut tampered = valid.clone().into_bytes();
    let last = tampered.len() - 1;
    tampered[last] = if tampered[last] == b'0' { b'1' } else { b'0' };
    let tampered = String::from_utf8(tampered).expect("utf8");
    assert!(!exceptions.is_exception_valid(file, guard, &tampered));

    // And the genuine token still validates.
    assert!(exceptions.is_exception_valid(file, guard, &valid));
}

#[test]
fn test_tokens_are_unique_across_targets() {
    let mut exceptions = ProjectExceptions::default();
    let mut seen: HashSet<String> = HashSet::new();

    for i in 0..25 {
        let file_str = format!("src/service_{i}.rs");
        let guard_str = format!("guard_{}", i % 5);
        let reason = format!("Authorized architecture exemption {i}");
        let entry = exceptions.insert_exception(Path::new(&file_str), &guard_str, &reason);
        assert_canonical(&entry.token);
        assert!(
            seen.insert(entry.token.clone()),
            "token collision for {file_str}/{guard_str}"
        );
        assert!(exceptions.is_exception_valid(Path::new(&file_str), &guard_str, &entry.token));
    }

    let stored: Vec<&String> = exceptions.exceptions.iter().map(|e| &e.token).collect();
    let unique: HashSet<&String> = stored.iter().copied().collect();
    assert_eq!(stored.len(), unique.len(), "no duplicate tokens in the store");
}
