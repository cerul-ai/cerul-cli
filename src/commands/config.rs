use anyhow::{bail, Result};
use colored::Colorize;

use crate::config::{self, UserConfig};

pub fn run_interactive() -> Result<()> {
    let mut cfg = config::load_config();

    eprintln!();
    eprintln!("  {}", "⚙️  Cerul Configuration".bold());
    eprintln!();

    // Images
    let current_images = if cfg.images { "on" } else { "off" };
    let input = config::read_line(&format!(
        "  Images in results  [{}] (on/off): ",
        current_images.dimmed()
    ))?;
    if !input.is_empty() {
        cfg.images = matches!(input.to_lowercase().as_str(), "on" | "true" | "yes" | "1");
    }

    // Max results
    let input = config::read_line(&format!(
        "  Default max results [{}] (1-50): ",
        cfg.max_results.to_string().dimmed()
    ))?;
    if !input.is_empty() {
        if let Ok(n) = input.parse::<u32>() {
            if (1..=50).contains(&n) {
                cfg.max_results = n;
            } else {
                eprintln!("  {}", "Invalid: must be 1-50, keeping current value.".yellow());
            }
        }
    }

    // Ranking mode
    let input = config::read_line(&format!(
        "  Default ranking mode [{}] (embedding/rerank): ",
        cfg.ranking_mode.dimmed()
    ))?;
    if !input.is_empty() {
        match input.to_lowercase().as_str() {
            "embedding" | "rerank" => cfg.ranking_mode = input.to_lowercase(),
            _ => eprintln!("  {}", "Invalid: must be embedding or rerank, keeping current value.".yellow()),
        }
    }

    // Include answer
    let current_answer = if cfg.include_answer { "on" } else { "off" };
    let input = config::read_line(&format!(
        "  Include AI answer  [{}] (on/off, costs 2 credits): ",
        current_answer.dimmed()
    ))?;
    if !input.is_empty() {
        cfg.include_answer = matches!(input.to_lowercase().as_str(), "on" | "true" | "yes" | "1");
    }

    config::save_config(&cfg)?;

    let path = config::config_file_path()?;
    eprintln!();
    eprintln!(
        "  {} Saved to {}",
        "✅".green(),
        path.display().to_string().dimmed()
    );
    eprintln!();

    Ok(())
}

pub fn run_list() -> Result<()> {
    let cfg = config::load_config();
    let path = config::config_file_path()?;

    eprintln!();
    eprintln!("  {}", "⚙️  Cerul Configuration".bold());
    eprintln!();
    eprintln!(
        "  {:<18}{}",
        "images".dimmed(),
        if cfg.images { "on".green().to_string() } else { "off".to_string() }
    );
    eprintln!(
        "  {:<18}{}",
        "max_results".dimmed(),
        cfg.max_results
    );
    eprintln!(
        "  {:<18}{}",
        "ranking_mode".dimmed(),
        cfg.ranking_mode
    );
    eprintln!(
        "  {:<18}{}",
        "include_answer".dimmed(),
        if cfg.include_answer { "on".green().to_string() } else { "off".to_string() }
    );
    eprintln!();
    eprintln!("  File: {}", path.display().to_string().dimmed());
    eprintln!();

    Ok(())
}

pub fn run_set(key: &str, value: &str) -> Result<()> {
    let mut cfg = config::load_config();

    match key {
        "images" => {
            cfg.images = matches!(value.to_lowercase().as_str(), "on" | "true" | "yes" | "1");
        }
        "max_results" | "max-results" => {
            let n: u32 = value.parse().map_err(|_| anyhow::anyhow!("Invalid number"))?;
            if !(1..=50).contains(&n) {
                bail!("max_results must be between 1 and 50");
            }
            cfg.max_results = n;
        }
        "ranking_mode" | "ranking-mode" => {
            if value != "embedding" && value != "rerank" {
                bail!("ranking_mode must be 'embedding' or 'rerank'");
            }
            cfg.ranking_mode = value.to_string();
        }
        "include_answer" | "include-answer" => {
            cfg.include_answer = matches!(value.to_lowercase().as_str(), "on" | "true" | "yes" | "1");
        }
        _ => bail!(
            "Unknown config key: {key}\n\nValid keys: images, max_results, ranking_mode, include_answer"
        ),
    }

    config::save_config(&cfg)?;
    eprintln!("  {} {} = {}", "✅".green(), key, value);

    Ok(())
}

pub fn run_reset() -> Result<()> {
    let cfg = UserConfig::default();
    config::save_config(&cfg)?;
    eprintln!();
    eprintln!("  {} Configuration reset to defaults.", "✅".green());
    eprintln!();
    Ok(())
}
