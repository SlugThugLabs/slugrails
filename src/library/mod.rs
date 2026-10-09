//! Loader, seeder, and creator for ~/.slugthuglabs/slugrails/tests/.

pub mod builtins;
pub mod catalog;

use crate::error::{Result, SlugRailsError};
use crate::library::builtins::get_builtin_guard_tests;
pub use crate::library::catalog::GuardCatalog;
use crate::types::GuardTestDefinition;
use crate::util::get_tests_dir;
use std::fs;
use std::path::{Path, PathBuf};

/// Ensures the global tests directory is initialized and seeded with built-in tests.
///
/// # Errors
///
/// Returns [`SlugRailsError::Io`] if the tests directory cannot be created or\
/// if built-in tests cannot be written to disk, or [`SlugRailsError::TomlParse`]\
/// if guard definitions cannot be serialized to JSON.
pub fn ensure_test_library_seeded() -> Result<GuardCatalog> {
    let tests_root = get_tests_dir();
    fs::create_dir_all(&tests_root).map_err(|e| SlugRailsError::Io {
        path: tests_root.clone(),
        source: e,
    })?;

    let catalog_file = tests_root.join("catalog.json");
    let builtins = get_builtin_guard_tests();

    // Fast Table of Contents verification:
    // If catalog.json already has all built-in tests on disk, do not rebuild!
    if catalog_file.exists()
        && let Ok(content) = fs::read_to_string(&catalog_file)
        && let Ok(catalog) = serde_json::from_str::<GuardCatalog>(&content)
    {
        let all_exist = builtins.iter().all(|b| {
            catalog.tests.get(&b.id).is_some_and(|e| e.file_path.exists())
        });
        if all_exist {
            return Ok(catalog);
        }
    }

    let mut definitions = Vec::new();

    for def in builtins {
        let category_dir = def.category.as_str();
        let slug_name = def.name.replace('_', "-");
        let file_name_final = format!("{slug_name}.guard.json");

        let target_dir = tests_root.join(category_dir);
        let _ = fs::create_dir_all(&target_dir);

        let test_file = target_dir.join(file_name_final);
        if !test_file.exists() {
            let json_str = serde_json::to_string_pretty(&def)?;
            let _ = fs::write(&test_file, json_str);
        }

        definitions.push((def, test_file));
    }

    // Load any user/custom tests from disk and update Table of Contents
    load_all_tests_and_save_catalog(&tests_root)
}

/// Scans the tests directory, loads all definitions, and writes catalog.json.
///
/// # Errors
///
/// Returns [`SlugRailsError::Io`] if the catalog cannot be written to disk,\
/// or [`SlugRailsError::TomlParse`] if guard definitions cannot be serialized to JSON.
pub fn load_all_tests_and_save_catalog(tests_root: &Path) -> Result<GuardCatalog> {
    let mut definitions = Vec::new();

    for entry in walkdir::WalkDir::new(tests_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            if path.file_name().is_some_and(|n| n == "catalog.json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(path)
                && let Ok(def) = serde_json::from_str::<GuardTestDefinition>(&content)
            {
                definitions.push((def, path.to_path_buf()));
            }
        }
    }

    let catalog = GuardCatalog::from_definitions(&definitions);
    let catalog_file = tests_root.join("catalog.json");
    let catalog_json = serde_json::to_string_pretty(&catalog)?;
    fs::write(&catalog_file, catalog_json).map_err(|e| SlugRailsError::Io {
        path: catalog_file,
        source: e,
    })?;

    Ok(catalog)
}

/// Creates a new custom guard test with duplicate prevention.
///
/// # Errors
///
/// Returns [`SlugRailsError::Io`] if the test file cannot be written to disk,\
/// [`SlugRailsError::TomlParse`] if the definition cannot be serialized to JSON,\
/// or [`SlugRailsError::InvalidGuardTest`] if a duplicate is detected without --force.
pub fn create_custom_guard_test(
    tests_root: &Path,
    def: &GuardTestDefinition,
    force: bool,
) -> Result<PathBuf> {
    let catalog_file = tests_root.join("catalog.json");
    let catalog = if catalog_file.exists() {
        let content = fs::read_to_string(&catalog_file).map_err(|e| SlugRailsError::Io {
            path: catalog_file.clone(),
            source: e,
        })?;
        serde_json::from_str::<GuardCatalog>(&content)?
    } else {
        GuardCatalog::default()
    };

    if !force && let Some(reason) = catalog.find_potential_duplicate(&def.name, &def.tags) {
        return Err(SlugRailsError::InvalidGuardTest {
            test_id: def.id.clone(),
            reason: format!("Duplicate guard prevented: {reason}. Use --force to override."),
        });
    }

    let target_dir = tests_root.join(&def.category);
    fs::create_dir_all(&target_dir).map_err(|e| SlugRailsError::Io {
        path: target_dir.clone(),
        source: e,
    })?;

    let slug_name = def.name.replace('_', "-");
    let target_file = target_dir.join(format!("{slug_name}.guard.json"));

    let json_str = serde_json::to_string_pretty(def)?;
    fs::write(&target_file, json_str).map_err(|e| SlugRailsError::Io {
        path: target_file.clone(),
        source: e,
    })?;

    // Refresh catalog.json
    load_all_tests_and_save_catalog(tests_root)?;

    Ok(target_file)
}
