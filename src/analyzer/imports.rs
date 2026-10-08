//! Language import extraction across Rust, Python, and TypeScript.

use super::tokenizer::tokenize_source;

/// Extracts imported modules from source lines across Rust (`use crate::...`),
/// Python (`import ...`, `from ... import`), and JS/TS (`import ... from '...'`).
#[must_use]
pub fn extract_imported_modules(source: &str) -> Vec<(usize, String)> {
    let mut imports = Vec::new();

    for line in tokenize_source(source) {
        let code = line.code_only.trim();

        // Rust: use crate::foo::bar; or use foo::bar;
        if let Some(stripped) = code.strip_prefix("use ") {
            let path_part = stripped.trim_matches(';').trim();
            imports.push((line.line_number, path_part.to_string()));
        }
        // Python: import foo or from foo import bar
        else if let Some(stripped) = code.strip_prefix("import ") {
            let mod_name = stripped.split_whitespace().next().unwrap_or("");
            imports.push((line.line_number, mod_name.to_string()));
        } else if let Some(stripped) = code.strip_prefix("from ") {
            let mod_name = stripped.split_whitespace().next().unwrap_or("");
            imports.push((line.line_number, mod_name.to_string()));
        }
    }

    imports
}
