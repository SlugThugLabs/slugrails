//! Request and parameter types for CodeGuards MCP tools.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateArchitectureRequest {
    /// Absolute path to the target project directory (default: current dir)
    pub project_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListGuardTestsRequest {
    /// Optional category filter (e.g. 'structural', 'complexity', 'hygiene', 'languages/rust')
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateGuardTestRequest {
    /// Unique test name (e.g. 'auth_required')
    pub name: String,
    /// Category (structural, complexity, hygiene, quality, languages/rust, custom/<ns>)
    pub category: String,
    /// Short explanation of what the guard checks
    pub summary: String,
    /// Search tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Known aliases
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Execution engine name
    pub engine: String,
    /// Actionable fix instruction when rule fails
    pub remediation: String,
    /// Override duplicate detection warnings
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddExceptionRequest {
    /// Project root directory
    pub project_path: String,
    /// Relative file path receiving exception
    pub file: String,
    /// Guard identifier (e.g. 'complexity/source-limits')
    pub guard_id: String,
    /// Specific architectural justification
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckProjectRequest {
    /// Absolute path to the target project directory (default: current dir)
    pub project_path: Option<String>,
    /// Check all source files instead of only git modified files (default: false)
    #[serde(default)]
    pub check_all: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProbeArchitectureRequest {
    /// High-level description of the system or component being planned
    pub description: String,
    /// Primary target programming language (e.g. 'rust', 'typescript', 'python')
    pub language: Option<String>,
    /// Optional target project directory (default: current dir)
    pub project_path: Option<String>,
    /// Automatically scaffold ARCHITECTURE.md, intent.json, and tests/slugrails.rs
    #[serde(default)]
    pub auto_scaffold: bool,
    /// Architectural style hint ('layered', 'hexagonal', 'cli')
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScaffoldHarnessRequest {
    /// Absolute path to target project directory (default: current dir)
    pub project_path: Option<String>,
}

