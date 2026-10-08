//! CLI subcommands and dispatcher.

pub mod args;
pub mod check;
pub mod exception;
pub mod fix;
pub mod hook;
pub mod init;
pub mod sarif;
pub mod serve;
pub mod validate;

pub use check::run_check;
pub use exception::run_exception;
pub use fix::auto_remediate_violations;
pub use hook::{install_hook, run_hook, uninstall_hook};
pub use init::{initialize_project, run_init};
pub use sarif::format_sarif;
pub use serve::run_serve;
pub use validate::run_validate;
