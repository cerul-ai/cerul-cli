use anyhow::Result;
use colored::Colorize;

use super::search::history;

pub fn run(limit: usize) -> Result<()> {
    let entries = history::read_recent(limit);

    if entries.is_empty() {
        eprintln!();
        eprintln!("  No search history yet. Try: {}", "cerul search \"sam altman agi\"".green());
        eprintln!();
        return Ok(());
    }

    eprintln!();
    eprintln!("  {}", "📋 Recent Searches".bold());
    eprintln!();

    for (timestamp, query, result_count) in &entries {
        eprintln!(
            "  {}  {}  {}",
            timestamp.dimmed(),
            query,
            format!("({result_count} results)").dimmed(),
        );
    }

    eprintln!();
    eprintln!(
        "  {}",
        format!("Showing last {} searches", entries.len()).dimmed()
    );
    eprintln!();

    Ok(())
}
