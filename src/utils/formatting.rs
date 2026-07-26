use colored::*;

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".bold().green(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".bold().cyan(), msg);
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".bold().yellow(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".bold().red(), msg);
}

pub fn format_diff(original: &str, transformed: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        "--- Original Prompt ---".dimmed(),
        original.yellow(),
        "--- Transformed Prompt ---".dimmed(),
        transformed.green().bold()
    )
}
