//! Manifest dependency scanner for Cargo.toml and package.json.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Dependencies declared in a project manifest file.
#[derive(Debug, Clone, Default)]
pub struct ManifestDependencies {
    pub direct_dependencies: BTreeSet<String>,
    pub dev_dependencies: BTreeSet<String>,
    pub build_dependencies: BTreeSet<String>,
}

impl ManifestDependencies {
    /// Checks if a dependency is present anywhere in the manifest.
    #[must_use]
    pub fn contains(&self, dep: &str) -> bool {
        self.direct_dependencies.contains(dep)
            || self.dev_dependencies.contains(dep)
            || self.build_dependencies.contains(dep)
    }
}

/// Scans Cargo.toml at the project root for declared dependencies.
#[must_use]
pub fn scan_cargo_manifest(project_root: &Path) -> Option<ManifestDependencies> {
    let manifest_path = project_root.join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;

    let mut direct = BTreeSet::new();
    let mut dev = BTreeSet::new();
    let mut build = BTreeSet::new();

    if let Some(deps) = value.get("dependencies").and_then(toml::Value::as_table) {
        for key in deps.keys() {
            direct.insert(key.clone());
        }
    }

    if let Some(deps) = value
        .get("dev-dependencies")
        .and_then(toml::Value::as_table)
    {
        for key in deps.keys() {
            dev.insert(key.clone());
        }
    }

    if let Some(deps) = value
        .get("build-dependencies")
        .and_then(toml::Value::as_table)
    {
        for key in deps.keys() {
            build.insert(key.clone());
        }
    }

    Some(ManifestDependencies {
        direct_dependencies: direct,
        dev_dependencies: dev,
        build_dependencies: build,
    })
}

/// Scans package.json at the project root for declared JavaScript/TypeScript dependencies.
#[must_use]
pub fn scan_package_json(project_root: &Path) -> Option<ManifestDependencies> {
    let pkg_path = project_root.join("package.json");
    let content = fs::read_to_string(&pkg_path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;

    let mut direct = BTreeSet::new();
    let mut dev = BTreeSet::new();

    if let Some(deps) = value
        .get("dependencies")
        .and_then(serde_json::Value::as_object)
    {
        for key in deps.keys() {
            direct.insert(key.clone());
        }
    }

    if let Some(deps) = value
        .get("devDependencies")
        .and_then(serde_json::Value::as_object)
    {
        for key in deps.keys() {
            dev.insert(key.clone());
        }
    }

    Some(ManifestDependencies {
        direct_dependencies: direct,
        dev_dependencies: dev,
        build_dependencies: BTreeSet::new(),
    })
}
