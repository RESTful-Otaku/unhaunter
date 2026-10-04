// tools/ghost_radio/src/console_ui.rs
use crate::data::GhostResponse;
use std::io::{self, Write};

pub fn display_ghost_options(ghost_responses: &[&str]) {
    println!("Available Ghosts:");
    for (i, key) in ghost_responses.iter().enumerate() {
        println!("{}. {}", i + 1, key);
    }
}

/// Reads a single line from stdin, returning `None` on EOF or I/O error.
fn read_line() -> Option<String> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => None, // EOF
        Ok(_) => Some(input),
        Err(_) => None,
    }
}

/// Prompts for a 1-based choice in `1..=count`. Returns `None` when stdin is
/// closed (EOF) so callers can exit cleanly instead of spinning forever.
pub fn prompt_choice(prompt: &str, count: usize) -> Option<usize> {
    loop {
        print!("{prompt}");
        io::stdout().flush().ok()?;
        let input = read_line()?;
        match input.trim().parse::<usize>() {
            Ok(choice) if choice >= 1 && choice <= count => return Some(choice - 1),
            Ok(_) => println!("Please enter a number between 1 and {count}."),
            Err(_) => println!("Invalid input. Please enter a number."),
        }
    }
}

pub fn get_player_phrase(phrases: &[String]) -> Option<String> {
    let idx = prompt_choice(
        &format!(
            "Enter a phrase (enter number 1-{}, or Ctrl-D to quit): ",
            phrases.len()
        ),
        phrases.len(),
    )?;
    phrases.get(idx).cloned()
}

pub fn display_ghost_response(response: &GhostResponse) {
    println!("Ghost response: {}", response.phrase);
}
