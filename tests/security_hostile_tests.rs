//! Hostile security and adversary resistance tests.

use codeguards_mcp::storage::ProjectExceptions;
use codeguards_mcp::util::{compute_exception_token, validate_safe_path};
use std::path::Path;

#[test]
fn test_sandbox_rejects_windows_unc_paths() {
    let unc_path = Path::new(r"\\attacker.evil.com\payload\exploit.rs");
    assert!(
        validate_safe_path(unc_path).is_err(),
        "Must reject Windows UNC network path"
    );

    let forward_unc = Path::new("//attacker.evil.com/payload/exploit.rs");
    assert!(
        validate_safe_path(forward_unc).is_err(),
        "Must reject forward slash UNC path"
    );
}

#[test]
fn test_sandbox_rejects_ntfs_alternate_data_streams() {
    let ads_path = Path::new("src/normal.rs:secret_stream");
    assert!(
        validate_safe_path(ads_path).is_err(),
        "Must reject NTFS alternate data stream syntax"
    );
}

#[test]
fn test_sandbox_rejects_dos_reserved_device_names() {
    let dos_names = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
    for name in dos_names {
        let path = Path::new(name);
        assert!(
            validate_safe_path(path).is_err(),
            "Must reject DOS device name {name}"
        );
        let ext_path = format!("src/{name}.rs");
        assert!(
            validate_safe_path(Path::new(&ext_path)).is_err(),
            "Must reject DOS device stem in {ext_path}"
        );
    }
}

#[test]
fn test_exceptions_rejects_forged_tokens() {
    let mut exceptions = ProjectExceptions::default();
    let file = Path::new("src/critical.rs");
    let guard_id = "complexity/source-limits";

    // Manually forge an exception entry with an unkeyed fake token
    let fake_entry = codeguards_mcp::types::ExceptionEntry {
        token: "sr_tok_fake12345678".to_string(),
        file: file.to_path_buf(),
        guard_id: guard_id.to_string(),
        reason: "arbitrary bypass".to_string(),
        granted_at: chrono::Utc::now().to_rfc3339(),
    };
    exceptions.exceptions.push(fake_entry);

    // Assert that validation detects the HMAC forgery and rejects it
    assert!(
        !exceptions.is_exception_valid(file, guard_id, "sr_tok_fake12345678"),
        "Must reject forged exception token not signed by central secret salt"
    );
}

#[test]
fn test_exceptions_accepts_valid_hmac_token() {
    let mut exceptions = ProjectExceptions::default();
    let file = Path::new("src/legit.rs");
    let guard_id = "hygiene/no-debug-prints";
    let reason = "legitimate architectural exception";

    // Compute genuine token using secret salt
    let valid_token = compute_exception_token(file, guard_id, reason);
    let entry = codeguards_mcp::types::ExceptionEntry {
        token: valid_token.clone(),
        file: file.to_path_buf(),
        guard_id: guard_id.to_string(),
        reason: reason.to_string(),
        granted_at: chrono::Utc::now().to_rfc3339(),
    };
    exceptions.exceptions.push(entry);

    assert!(
        exceptions.is_exception_valid(file, guard_id, &valid_token),
        "Must accept cryptographically verified token"
    );
}
