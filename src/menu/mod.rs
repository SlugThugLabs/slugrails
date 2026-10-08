//! Interactive setup menu — walks a human through installing the binary,
//! connecting to AI agents, seeding the guard-test library, or running the MCP server.

mod render;
mod steps;

use crate::style::Style;
use std::io::{BufRead as _, Write as _};

pub use render::render_menu;
pub use steps::{confirm_serve, connect_step, guard_catalog_step, install_step, other_client_step};

/// Reads one menu choice from stdin. Unparseable input maps to `0` so the
/// caller's `_` arm reports it as invalid rather than panicking.
fn read_choice() -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let mut line = String::new();
    let bytes_read = std::io::stdin().lock().read_line(&mut line)?;
    if bytes_read == 0 {
        // EOF reached
        return Ok(None);
    }
    let choice = line.trim().parse().unwrap_or(0);
    Ok(Some(choice))
}

/// Runs the interactive setup menu. Returns `Ok(true)` if user chose to run `serve`.
///
/// # Errors
///
/// Returns an error if the menu prompt cannot be flushed to stdout or if
/// reading the user's choice from stdin fails.
pub fn run_menu() -> Result<bool, Box<dyn std::error::Error>> {
    loop {
        let style = Style::stdout();
        print!("{}", render_menu(style));
        std::io::stdout().flush()?;
        let Some(choice) = read_choice()? else {
            return Ok(false); // Exit on EOF
        };
        match choice {
            1 => install_step(),
            2 => connect_step(read_choice),
            3 => other_client_step(),
            4 => guard_catalog_step(),
            5 => {
                if confirm_serve()? {
                    return Ok(true);
                }
            }
            6 => return Ok(false),
            _ => {
                println!("\nInvalid choice; pick a number from the list.");
                continue;
            }
        }
        println!();
    }
}
