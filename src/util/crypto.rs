//! Cryptographic operations, exception token generation, and secret salt management.

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static SECRET_SALT_CACHE: OnceLock<[u8; 32]> = OnceLock::new();

/// Retrieves or initializes a local private HMAC salt secret in ~/.slugthuglabs/slugrails/.secret.key.
///
/// Uses [`OnceLock`] to ensure the salt is read from disk only once per process lifecycle.
#[must_use]
pub fn get_or_init_secret_salt() -> [u8; 32] {
    *SECRET_SALT_CACHE.get_or_init(load_or_generate_secret_salt)
}

fn load_or_generate_secret_salt() -> [u8; 32] {
    let home = crate::util::platform::get_slugrails_home();
    let key_path = home.join(".secret.key");

    if let Ok(bytes) = fs::read(&key_path)
        && bytes.len() == 32
    {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        return arr;
    }

    let _ = fs::create_dir_all(&home);
    let random_key = uuid::Uuid::new_v4();
    let mut hasher = blake3::Hasher::new();
    hasher.update(random_key.as_bytes());
    if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        hasher.update(user.as_bytes());
    }
    hasher.update(chrono::Utc::now().to_rfc3339().as_bytes());
    hasher.update(b"slugrails-salt-key-v3");
    let key_bytes = *hasher.finalize().as_bytes();

    if let Err(e) = fs::write(&key_path, key_bytes) {
        tracing::warn!(
            "Failed to persist secret salt to {}: {e}",
            key_path.display()
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
    }
    key_bytes
}

/// Computes an HMAC token using a specified date string.
#[must_use]
pub fn compute_exception_token_with_date(
    file: &Path,
    guard_id: &str,
    reason: &str,
    date_str: &str,
) -> String {
    let salt = get_or_init_secret_salt();
    let mut hasher = blake3::Hasher::new_keyed(&salt);
    hasher.update(file.to_string_lossy().as_bytes());
    hasher.update(guard_id.as_bytes());
    hasher.update(reason.as_bytes());
    hasher.update(date_str.as_bytes());
    let hash = hasher.finalize();
    let bytes = hash.as_bytes();
    let num = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) % 90000 + 10000;
    num.to_string()
}

/// Computes a secure, salted 16-character hex verification token for an exception.
#[must_use]
pub fn compute_exception_token(file: &Path, guard_id: &str, reason: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d").to_string();
    compute_exception_token_with_date(file, guard_id, reason, &timestamp)
}

/// Computes the raw 32-byte BLAKE3 hash of arbitrary byte data.
#[must_use]
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Computes the full 64-character hex BLAKE3 hash of arbitrary byte data.
#[must_use]
pub fn blake3_hash_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}
