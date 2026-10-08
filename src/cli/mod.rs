//! CLI subcommands and dispatcher.

pub mod check;
pub mod exception;
pub mod init;
pub mod sarif;
pub mod serve;
pub mod validate;

pub use check::run_check;
pub use exception::run_exception;
pub use init::{initialize_project, run_init};
pub use sarif::format_sarif;
pub use serve::run_serve;
pub use validate::run_validate;
