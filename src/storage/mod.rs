//! Project storage manager and user-authorized exception store.

pub mod baseline;
pub use baseline::{
    compute_contract_hash, get_contract_baseline_path, save_contract_baseline,
    verify_contract_baseline,
};

use crate::error::{Result, SlugRailsError};
use crate::types::ExceptionEntry;
use crate::util::{
    compute_exception_token, compute_exception_token_with_date, get_project_storage_dir,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Project-specific exceptions registry stored at ~/.slugthuglabs/slugrails/projects/<id>/exceptions.json
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectExceptions {
    pub project_path: PathBuf,
    pub version: String,
    pub exceptions: Vec<ExceptionEntry>,
}

impl ProjectExceptions {
    /// Loads the exceptions registry for a given project, recovering if corrupt.
    ///
    /// # Errors
    ///
    /// Returns [`SlugRailsError::Io`] if the exceptions file cannot be read from disk.
    pub fn load(project_path: &Path) -> Result<Self> {
        let storage_dir = get_project_storage_dir(project_path);
        let file_path = storage_dir.join("exceptions.json");

        if !file_path.exists() {
            return Ok(Self {
                project_path: project_path.to_path_buf(),
                version: "1.0.0".to_string(),
                exceptions: Vec::new(),
            });
        }

        let content = fs::read_to_string(&file_path).map_err(|e| SlugRailsError::Io {
            path: file_path.clone(),
            source: e,
        })?;

        match serde_json::from_str::<Self>(&content) {
            Ok(registry) => Ok(registry),
            Err(_) => {
                let backup_name = format!("exceptions.json.corrupt.{}", Utc::now().timestamp());
                let _ = fs::rename(&file_path, storage_dir.join(backup_name));
                Ok(Self {
                    project_path: project_path.to_path_buf(),
                    version: "1.0.0".to_string(),
                    exceptions: Vec::new(),
                })
            }
        }
    }

    /// Atomically saves the exceptions registry to disk using a temporary file rename.
    ///
    /// # Errors
    ///
    /// Returns [`SlugRailsError::Io`] if the exceptions file cannot be written to disk.
    pub fn save(&self) -> Result<PathBuf> {
        let storage_dir = get_project_storage_dir(&self.project_path);
        fs::create_dir_all(&storage_dir).map_err(|e| SlugRailsError::Io {
            path: storage_dir.clone(),
            source: e,
        })?;

        let file_path = storage_dir.join("exceptions.json");
        let tmp_path = storage_dir.join(format!(
            "exceptions.json.tmp.{}.{}", std::process::id(), Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&tmp_path, content).map_err(|e| SlugRailsError::Io {
            path: tmp_path.clone(),
            source: e,
        })?;

        #[cfg(windows)]
        let _ = fs::remove_file(&file_path);

        fs::rename(&tmp_path, &file_path).map_err(|e| SlugRailsError::Io {
            path: file_path.clone(),
            source: e,
        })?;

        Ok(file_path)
    }

    /// Pure in-memory insertion of an authorized exception token without disk I/O.
    pub fn insert_exception(
        &mut self, file: &Path, guard_id: &str, reason: &str,
    ) -> ExceptionEntry {
        let token = compute_exception_token(file, guard_id, reason);
        self.exceptions.retain(|e| e.token != token);

        let entry = ExceptionEntry {
            token,
            file: file.to_path_buf(),
            guard_id: guard_id.to_string(),
            reason: reason.to_string(),
            granted_at: chrono::Utc::now().to_rfc3339(),
        };

        self.exceptions.push(entry.clone());
        entry
    }

    /// Adds a user-authorized exception and persists it to disk.
    ///
    /// # Errors
    ///
    /// Returns [`SlugRailsError::Io`] if the exceptions file cannot be written to disk.
    pub fn add_exception(
        &mut self, file: &Path, guard_id: &str, reason: &str,
    ) -> Result<ExceptionEntry> {
        let entry = self.insert_exception(file, guard_id, reason);
        self.save()?;
        Ok(entry)
    }

    /// Pure in-memory removal of an exception token without disk I/O.
    pub fn remove_exception(&mut self, token: &str) -> bool {
        let before_len = self.exceptions.len();
        self.exceptions.retain(|e| e.token != token);
        self.exceptions.len() < before_len
    }

    /// Revokes an exception by token and persists the change to disk.
    ///
    /// # Errors
    ///
    /// Returns [`SlugRailsError::Io`] if the exceptions file cannot be written to disk.
    pub fn revoke(&mut self, token: &str) -> Result<bool> {
        let removed = self.remove_exception(token);
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    /// Validates whether a file and guard violation is covered by an active exception token.
    #[must_use]
    pub fn is_exception_valid(&self, file: &Path, guard_id: &str, token: &str) -> bool {
        self.is_exception_valid_with_ttl(file, guard_id, token, 30)
    }

    /// Validates whether an exception token is active within `ttl_days`.
    #[must_use]
    pub fn is_exception_valid_with_ttl(
        &self, file: &Path, guard_id: &str, token: &str, ttl_days: i64,
    ) -> bool {
        self.exceptions.iter().any(|e| {
            let norm_file = file.to_string_lossy().replace('\\', "/");
            let norm_e_file = e.file.to_string_lossy().replace('\\', "/");
            let path_matches = norm_file == norm_e_file
                || (norm_e_file.contains('/') && norm_file.ends_with(&norm_e_file));

            if !path_matches
                || !e.guard_id.eq_ignore_ascii_case(guard_id)
                || is_token_expired(&e.granted_at, ttl_days)
            {
                return false;
            }

            let date_part = e.granted_at.get(..10).unwrap_or("");
            let exp = compute_exception_token_with_date(&e.file, &e.guard_id, &e.reason, date_part);
            let legacy_exp = compute_exception_token(&e.file, &e.guard_id, &e.reason);
            e.token == token && (e.token == exp || e.token == legacy_exp)
        })
    }
}

/// Checks if a token is older than `ttl_days` (default 30 days).
fn is_token_expired(granted_at: &str, ttl_days: i64) -> bool {
    DateTime::parse_from_rfc3339(granted_at)
        .map_or(true, |t| t + chrono::Duration::days(ttl_days) < Utc::now())
}
