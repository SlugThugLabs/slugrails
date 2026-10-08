//! CLI subcommands and dispatcher.

pub mod check;
pub mod exception;
pub mod serve;
pub mod validate;

pub use check::run_check;
pub use exception::run_exception;
pub use serve::run_serve;
pub use validate::run_validate;
