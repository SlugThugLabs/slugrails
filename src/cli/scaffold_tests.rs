//! Scaffolds a standalone integration test harness in tests/slugrails.rs.

use crate::cli::args::extract_root;
use std::path::Path;
use std::process::ExitCode;

pub use crate::contract::scaffold::TEST_HARNESS_TEMPLATE;

/// Writes `tests/slugrails.rs` to the project root.
pub fn run_scaffold_tests(args: impl Iterator<Item = String>) -> ExitCode {
    let args_vec: Vec<String> = args.collect();
    if crate::cli::args::has_help_flag(&args_vec) {
        println!("Usage: slugrails scaffold-tests [--root <path>]\n\nGenerates tests/slugrails.rs integration test harness for native cargo test enforcement.");
        return ExitCode::SUCCESS;
    }
    let root = extract_root(&args_vec);

    match scaffold_tests_harness(&root) {
        Ok(path) => {
            println!("Successfully scaffolded SlugRails harness: {}", path.display());
            println!("Run 'cargo test --test slugrails' to execute architectural enforcement.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Failed to scaffold test harness: {e}");
            ExitCode::from(1)
        }
    }
}

/// Generates tests/slugrails.rs in the specified project directory.
pub fn scaffold_tests_harness(root: &Path) -> std::io::Result<std::path::PathBuf> {
    crate::contract::scaffold_tests_harness(root)
}
