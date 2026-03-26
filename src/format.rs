use colored::Colorize;

pub fn format_result(command: &str, explanation: &str, show_explanation: bool) {
    if !command.is_empty() {
        println!("\n  {}", command.green().bold());
    }

    if show_explanation && !explanation.is_empty() {
        println!("  {}", "─".repeat(40).dimmed());
        for line in explanation.lines() {
            println!("  {}", line);
        }
    }

    println!();
}

pub fn format_raw(command: &str) {
    println!("{}", command);
}

pub fn format_talk(response: &str) {
    println!();
    for line in response.lines() {
        println!("  {}", line);
    }
    println!();
}
