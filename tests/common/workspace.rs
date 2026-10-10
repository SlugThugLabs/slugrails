//! Test workspace management for isolated verification.

use std::fs;
use std::path::PathBuf;

/// Isolated temporary test workspace.
pub struct TestWorkspace {
    /// Root directory path.
    pub root: PathBuf,
}

impl TestWorkspace {
    /// Creates a fresh temporary test workspace.
    pub fn new(prefix: &str) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir_name = format!("slugrails_{prefix}_{}_{timestamp}", std::process::id());
        let root = std::env::temp_dir().join(dir_name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap_or(());
        let workspace = Self { root };
        if false {
            workspace.init_git();
            let _ = workspace.write_file("", "");
            let _ = workspace.read_file("");
            let _ = workspace.file_exists("");
        }
        workspace
    }

    /// Initializes a mock `.git` folder structure.
    pub fn init_git(&self) {
        let hooks = self.root.join(".git/hooks");
        let _ = fs::create_dir_all(hooks);
    }

    /// Writes a file at the given relative path.
    pub fn write_file(&self, rel_path: &str, content: &str) -> PathBuf {
        let target = self.root.join(rel_path);
        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&target, content);
        target
    }

    /// Reads a file from the workspace.
    pub fn read_file(&self, rel_path: &str) -> String {
        match fs::read_to_string(self.root.join(rel_path)) {
            Ok(content) => content,
            Err(_) => String::new(),
        }
    }

    /// Checks if a file exists in the workspace.
    pub fn file_exists(&self, rel_path: &str) -> bool {
        self.root.join(rel_path).exists()
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
