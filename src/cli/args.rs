//! Unified CLI argument parsing helpers.

use std::path::PathBuf;

/// Extracts the `--root` / `-r` / `--project` / `-p` path from command arguments if provided.
#[must_use]
pub fn extract_root(args: &[String]) -> PathBuf {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if let Some(r) = arg
            .strip_prefix("--root=")
            .or_else(|| arg.strip_prefix("-r="))
            .or_else(|| arg.strip_prefix("--project="))
            .or_else(|| arg.strip_prefix("-p="))
        {
            return PathBuf::from(r);
        }
        if (arg == "--root" || arg == "-r" || arg == "--project" || arg == "-p")
            && let Some(next) = iter.next()
        {
            return PathBuf::from(next);
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Checks whether any of the given boolean flags (e.g. `["--all", "-a"]`) are present.
#[must_use]
pub fn has_flag(args: &[String], flags: &[&str]) -> bool {
    args.iter().any(|arg| flags.contains(&arg.as_str()))
}

/// Checks whether help was requested (`--help`, `-h`, or `help`).
#[must_use]
pub fn has_help_flag(args: &[String]) -> bool {
    has_flag(args, &["--help", "-h", "help"])
}

/// Extracts a string option with standard formats (`--key=val`, `--key val`, `-k=val`, `-k val`).
#[must_use]
pub fn extract_option(args: &[String], long: &str, short: Option<&str>) -> Option<String> {
    let long_prefix = format!("{long}=");
    let short_prefix = short.map(|s| format!("{s}="));

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if let Some(val) = arg.strip_prefix(&long_prefix) {
            return Some(val.to_string());
        }
        if let Some(ref sp) = short_prefix
            && let Some(val) = arg.strip_prefix(sp)
        {
            return Some(val.to_string());
        }
        if (arg == long || short.is_some_and(|s| s == arg))
            && let Some(next) = iter.next()
        {
            return Some(next.to_string());
        }
    }
    None
}
