//! Auto-remediation engine for self-healing guard violations.

use crate::types::Violation;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Automatically remediates supported violations (e.g. hygiene/no-debug-prints).
///
/// Returns the number of fixed violations.
#[must_use]
pub fn auto_remediate_violations(project_root: &Path, violations: &[Violation]) -> usize {
    let mut fixed_count = 0;
    let mut affected_files: HashSet<&Path> = HashSet::new();

    for v in violations {
        if v.guard_id == "hygiene/no-debug-prints" {
            affected_files.insert(&v.file);
        }
    }

    for rel_file in affected_files {
        let full_path = if rel_file.is_absolute() {
            rel_file.to_path_buf()
        } else {
            project_root.join(rel_file)
        };

        if let Ok(content) = fs::read_to_string(&full_path) {
            let mut modified = false;
            let mut new_lines = Vec::new();

            for line in content.lines() {
                let trimmed = line.trim();
                // Strip standalone debug print statements
                if (trimmed.starts_with("dbg!(") && trimmed.ends_with(");"))
                    || (trimmed.starts_with("console.log(") && trimmed.ends_with(");"))
                {
                    modified = true;
                    fixed_count += 1;
                } else {
                    new_lines.push(line);
                }
            }

            if modified {
                let mut out = new_lines.join("\n");
                if content.ends_with('\n') {
                    out.push('\n');
                }
                if fs::write(&full_path, out).is_ok() {
                    println!(
                        "[SLUGRAILS-FIX] Cleaned debug statements in {}",
                        rel_file.display()
                    );
                }
            }
        }
    }

    fixed_count
}
