#![allow(warnings, clippy::all, clippy::pedantic)]

//! Boundary tests for `resolve_root` / `validate_safe_path` (audit SR-S02 / MT-11).
//!
//! `validate_safe_path(root, path)` enforces containment inside an already-resolved
//! root. It cannot authorize the root itself; that limitation is documented in the
//! audit's IMPLEMENTATION_PROGRESS and is not asserted here.

use std::path::Path;

use slugrails::error::SlugRailsError;
use slugrails::util::{resolve_root, validate_safe_path};

#[test]
fn existing_paths_outside_the_root_are_rejected() {
    // /etc/passwd exists; it is outside a /tmp-scoped root.
    assert!(
        matches!(
            validate_safe_path(Path::new("/tmp"), Path::new("/etc/passwd")),
            Err(SlugRailsError::SandboxViolation { .. })
        ),
        "/etc/passwd must be rejected when the root is /tmp"
    );
    assert!(
        matches!(
            validate_safe_path(Path::new("/tmp"), Path::new("/")),
            Err(SlugRailsError::SandboxViolation { .. })
        ),
        "the filesystem root must not be accepted as a child path"
    );
}

#[test]
fn parent_traversal_cannot_escape_the_root() {
    let root = Path::new("/tmp/slugrails-boundary/root");
    for candidate in [
        "/tmp/slugrails-boundary/root/../outside.rs",
        "/tmp/slugrails-boundary/root/../../etc/passwd",
        "../outside.rs",
    ] {
        assert!(
            matches!(
                validate_safe_path(root, Path::new(candidate)),
                Err(SlugRailsError::SandboxViolation { .. })
            ),
            "{candidate} must not escape the root"
        );
    }
}

#[test]
fn paths_inside_a_nonexistent_root_are_resolved_lexically_and_allowed() {
    let root = Path::new("/tmp/slugrails-boundary/root");
    let inside = root.join("src/not_created_yet.rs");
    let resolved = validate_safe_path(root, &inside).expect("in-root path must be allowed");
    assert!(resolved.starts_with(root), "resolved path must stay in the root");
}

#[cfg(unix)]
#[test]
fn symlinks_that_escape_the_root_are_rejected() {
    use std::os::unix::fs::symlink;

    let base = tempfile::tempdir().expect("tempdir");
    let root = base.path().join("root");
    std::fs::create_dir_all(&root).expect("create root");
    let inside = root.join("inside.rs");
    std::fs::write(&inside, "// ok\n").expect("write inside");
    assert!(
        validate_safe_path(&root, &inside).is_ok(),
        "a real file inside the root must be allowed"
    );

    let escape = root.join("escape.rs");
    symlink("/etc/passwd", &escape).expect("symlink");
    assert!(
        matches!(
            validate_safe_path(&root, &escape),
            Err(SlugRailsError::SandboxViolation { .. })
        ),
        "a symlink resolving outside the root must be rejected"
    );

    let outside_link = base.path().join("outside_link.rs");
    symlink(&inside, &outside_link).expect("symlink");
    assert!(
        matches!(
            validate_safe_path(&root, &outside_link),
            Err(SlugRailsError::SandboxViolation { .. })
        ),
        "a path outside the root must be rejected even if it points inside"
    );
}

#[test]
fn sensitive_locations_are_rejected_regardless_of_root() {
    // .ssh is rejected by the deny-list even though it is inside the root.
    let root = Path::new("/root");
    assert!(
        matches!(
            validate_safe_path(root, Path::new("/root/.ssh/id_rsa")),
            Err(SlugRailsError::SandboxViolation { .. })
        ),
        "sensitive locations must be rejected on the deny-list, not just containment"
    );
    assert!(
        matches!(
            resolve_root(Path::new("/etc/shadow")),
            Err(SlugRailsError::SandboxViolation { .. })
        ),
        "resolve_root must reject sensitive roots"
    );
}

#[test]
fn resolve_root_accepts_a_normal_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let resolved = resolve_root(dir.path()).expect("a normal directory is a valid root");
    assert!(resolved.is_absolute());
}
