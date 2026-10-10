//! Negative fixture: Swallowed errors via let _ = expression.

pub fn execute_unhandled_task() {
    let _ = std::fs::remove_file("nonexistent.txt");
}
