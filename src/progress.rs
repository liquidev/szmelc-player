//! Progress reporting utilities.

use colored::Colorize;

/// Reports a task.
pub fn task(name: &str) {
   println!("{}", name.bold());
}
