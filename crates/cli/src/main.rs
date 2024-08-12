use owo_colors::OwoColorize;

fn main() {
    println!(
        "{}   {}\n\n\n{}{} <command> {}",
        " _    ___\n| |  / (_)___  ___\n| | / / / __ \\/ _ \\\n| |/ / / / / /  __/\n|___/_/_/ /_/\\___/".green().bold(),
        "v0.0.3".bright_black(),
        "Usage: ".bright_white().bold(),
        "vine".green(),
        "[...flags]".cyan(),
    )
}