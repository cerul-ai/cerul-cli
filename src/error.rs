use colored::Colorize;

pub fn exit_with_error(err: &anyhow::Error) -> ! {
    let message = format!("{err}");
    let lines: Vec<&str> = message.lines().collect();

    eprintln!();
    if let Some(first) = lines.first() {
        eprintln!("  {} {}", "✖".red(), first.bold());
    }
    for line in lines.iter().skip(1) {
        eprintln!("  {line}");
    }
    eprintln!();

    std::process::exit(1);
}
