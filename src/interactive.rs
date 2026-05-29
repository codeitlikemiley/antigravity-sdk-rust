//! Interactive CLI loop for conversational agent sessions.
//!
//! Provides [`run_interactive_loop()`] as a convenience for building REPL-style
//! agent interfaces. Matches Python SDK's `utils/interactive.py`.

use crate::agent::{Agent, Started};
use crate::types::ChatResponse;
use anyhow::Result;

/// Runs an interactive read-eval-print loop with the given started agent.
///
/// Reads user input from stdin, sends it to the agent via `chat()`,
/// and prints the response. Type `exit` or `quit` to end the session.
///
/// # Example
/// ```no_run
/// use antigravity_sdk_rust::agent::Agent;
/// use antigravity_sdk_rust::interactive::run_interactive_loop;
///
/// #[tokio::main]
/// async fn main() -> Result<(), anyhow::Error> {
///     let agent = Agent::builder().allow_all().build().start().await?;
///     run_interactive_loop(&agent).await?;
///     agent.stop().await?;
///     Ok(())
/// }
/// ```
pub async fn run_interactive_loop(agent: &Agent<Started>) -> Result<()> {
    use std::io::{self, BufRead, Write};

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        write!(stdout, "\n> ")?;
        stdout.flush()?;

        let mut input = String::new();
        let bytes_read = stdin.lock().read_line(&mut input)?;

        // EOF (Ctrl-D)
        if bytes_read == 0 {
            println!("\nGoodbye!");
            break;
        }

        let trimmed = input.trim();

        // Exit commands
        if matches!(trimmed.to_lowercase().as_str(), "exit" | "quit" | "q") {
            println!("Goodbye!");
            break;
        }

        // Skip empty input
        if trimmed.is_empty() {
            continue;
        }

        // Send to agent
        match agent.chat(trimmed).await {
            Ok(response) => {
                print_response(&response);
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }

    Ok(())
}

/// Formats and prints an agent response to stdout.
fn print_response(response: &ChatResponse) {
    if !response.thinking.is_empty() {
        println!("\n💭 {}", response.thinking);
    }
    if !response.text.is_empty() {
        println!("\n{}", response.text);
    }
    // Print usage stats
    let usage = &response.usage_metadata;
    if usage.total_token_count > 0 {
        println!(
            "\n📊 Tokens: {} prompt, {} response, {} total",
            usage.prompt_token_count, usage.candidates_token_count, usage.total_token_count
        );
    }
}
