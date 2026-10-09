//! Cryptographic operations, exception token generation, and secret salt management.

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static SECRET_SALT_CACHE: OnceLock<[u8; 32]> = OnceLock::new();

/// Retrieves or initializes the local keyed-BLAKE3 salt secret in
/// `~/.slugthuglabs/slugrails/.secret.key`.
///
/// Uses [`OnceLock`] to ensure the salt is read from disk only once per process lifecycle.
#[must_use]
pub fn get_or_init_secret_salt() -> [u8; 32] {
    *SECRET_SALT_CACHE.get_or_init(load_or_generate_secret_salt)
}

fn parse_hex_salt(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut arr = [0u8; 32];
    for (i, byte) in arr.iter_mut().enumerate() {
        *byte = u8::from_str_radix(s.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(arr)
}

fn load_or_generate_secret_salt() -> [u8; 32] {
    if let Ok(salt_str) = std::env::var("SLUGRAILS_SECRET_SALT") {
        let trimmed = salt_str.trim();
        if !trimmed.is_empty() {
            if let Some(bytes) = parse_hex_salt(trimmed) {
                return bytes;
            }
            return *blake3::hash(trimmed.as_bytes()).as_bytes();
        }
    }

    let home = crate::util::platform::get_slugrails_home();
    let key_path = home.join(".secret.key");

    if let Ok(bytes) = fs::read(&key_path)
        && bytes.len() == 32
    {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        return arr;
    }

    if let Err(e) = fs::create_dir_all(&home) {
        tracing::warn!("Failed to create salt directory {}: {e}", home.display());
    }
    let random_key = uuid::Uuid::new_v4();
    let mut hasher = blake3::Hasher::new();
    hasher.update(random_key.as_bytes());
    if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        hasher.update(user.as_bytes());
    }
    hasher.update(chrono::Utc::now().to_rfc3339().as_bytes());
    hasher.update(b"slugrails-salt-key-v3");
    let key_bytes = *hasher.finalize().as_bytes();

    let mut open_opts = fs::OpenOptions::new();
    open_opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        open_opts.mode(0o600);
    }
    match open_opts.open(&key_path) {
        Ok(mut f) => {
            use std::io::Write;
            if let Err(e) = f.write_all(&key_bytes) {
                tracing::warn!(
                    "Failed to persist secret salt to {}: {e}",
                    key_path.display()
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                "Failed to create secret salt file {}: {e}",
                key_path.display()
            );
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600)) {
            tracing::warn!("Failed to restrict permissions on {}: {e}", key_path.display());
        }
    }
    key_bytes
}

fn hash_exception_inputs(
    file: &Path,
    guard_id: &str,
    reason: &str,
    date_str: &str,
) -> blake3::Hash {
    let salt = get_or_init_secret_salt();
    let mut hasher = blake3::Hasher::new_keyed(&salt);
    let norm_path = file.to_string_lossy().replace('\\', "/");
    hasher.update(norm_path.as_bytes());
    hasher.update(guard_id.as_bytes());
    hasher.update(reason.as_bytes());
    hasher.update(date_str.as_bytes());
    hasher.finalize()
}

/// Computes the canonical 128-bit exception token (`sr_tok_` + 32 hex chars).
///
/// The token is a keyed-BLAKE3 MAC over `(path, guard, reason, grant date)` using the
/// local salt secret. See `.planning/adr/0002-exception-token-contract.md`.
#[must_use]
pub fn compute_exception_token_with_date(
    file: &Path,
    guard_id: &str,
    reason: &str,
    date_str: &str,
) -> String {
    let hash = hash_exception_inputs(file, guard_id, reason, date_str);
    format!("sr_tok_{}", &hash.to_hex()[..32])
}

/// Computes a secure, salted 128-bit hex verification token for an exception.
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
