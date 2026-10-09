//! Terminal text reporting formatting for `SlugRails` CLI checks.

use crate::types::GuardReport;

/// Prints the formatted human-readable terminal report for check violations.
pub fn print_text_report(report: &GuardReport) {
    if !report.unevaluated_tests.is_empty() {
        println!(
            "\x1b[33mwarning\x1b[0m: {} guard test(s) in contract were not found in library catalog:",
            report.unevaluated_tests.len()
        );
        for rule in &report.unevaluated_tests {
            println!("   - {rule}");
        }
    }

    if report.truncated || report.unreadable_entries > 0 || !report.skipped.is_empty() {
        println!(
            "\x1b[33mwarning\x1b[0m: scan incomplete — {} file(s) skipped, {} unreadable entry(ies), truncated={}",
            report.skipped.len(),
            report.unreadable_entries,
            report.truncated
        );
        for entry in &report.skipped {
            println!("   - {:?}: {}", entry.reason, entry.file.display());
        }
    }

    if report.is_pass() {
        println!(
            "[SLUGRAILS-PASS] {} files verified across {} active guard rules in {}ms.",
            report.total_files_checked,
            report.passed_tests.len(),
            report.duration_ms
        );
    } else if report.violations.is_empty() {
        println!(
            "\n\x1b[1;31m[SLUGRAILS-INCOMPLETE]\x1b[0m no violations in the inspected files, but the scan was partial — not a verified pass."
        );
    } else {
        println!(
            "\n\x1b[1;31merror\x1b[0m: architecture guard violations detected ({})",
            report.violations.len()
        );
        for v in &report.violations {
            let line_num = v.line.unwrap_or(1);
            let snippet = std::fs::read_to_string(&v.file)
                .ok()
                .and_then(|c| {
                    c.lines()
                        .nth(line_num.saturating_sub(1))
                        .map(|s| s.trim_end().to_string())
                })
                .unwrap_or_default();

            println!(
                "\n  \x1b[1;31m[{}]\x1b[0m \x1b[1m{}\x1b[0m",
                v.guard_id, v.message
            );
            println!("    \x1b[36m-->\x1b[0m {}:{line_num}", v.file.display());
            println!("     \x1b[36m|\x1b[0m");
            if snippet.is_empty() {
                println!(" {line_num:3} \x1b[36m|\x1b[0m \x1b[31m^^^^ violation coordinate\x1b[0m");
            } else {
                println!(" {line_num:3} \x1b[36m|\x1b[0m {snippet}");
                println!(
                    "     \x1b[36m|\x1b[0m \x1b[1;31m^\x1b[0m \x1b[31mviolation detected here\x1b[0m"
                );
            }
            println!("     \x1b[36m|\x1b[0m");
            if let Some(fix) = &v.fix_suggestion {
                println!("     \x1b[32m= help:\x1b[0m {fix}");
            }
            println!(
                "     \x1b[33m= bypass:\x1b[0m slugrails exception add {} {} --reason=\"...\"",
                v.file.display(),
                v.guard_id
            );
        }
        println!(
            "\n\x1b[1;31m[SLUGRAILS-BLOCKED] Failed with {} violation(s)\x1b[0m",
            report.violations.len()
        );
    }
}
