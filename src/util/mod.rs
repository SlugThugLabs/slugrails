//! Sandboxing and utility functions for SlugRails.

pub mod crypto;
pub mod paths;
pub mod platform;

pub use crypto::{
    blake3_hash, blake3_hash_hex, compute_exception_token,
    compute_exception_token_with_date, get_or_init_secret_salt,
};
pub use paths::{
    get_project_plan_dir, get_project_storage_dir, hash_project_path,
    is_windows_dos_device, normalize_path, validate_safe_path,
    RESERVED_DOS_NAMES,
};
pub use platform::{
    get_binary_name, get_codeguards_home, get_slugrails_home, get_slugthug_home,
    get_tests_dir,
};
