use anyhow::{bail, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

use crate::config::{self, UserConfig};

pub fn run_interactive() -> Result<()> {
    let mut cfg = config::load_config();

    eprintln!();
    eprintln!("  {}", "⚙️  Cerul Configuration".bold());
    eprintln!();

    loop {
        let items = vec![
            format!(
                "Images in results     {}",
                if cfg.images {
                    "on".green().to_string()
                } else {
                    "off".dimmed().to_string()
                }
            ),
            format!(
                "Default max results   {}",
                cfg.max_results.to_string().bold()
            ),
            format!(
                "Default ranking mode  {}",
                cfg.ranking_mode.bold()
            ),
            format!(
                "Include AI answer     {}",
                if cfg.include_answer {
                    "on".green().to_string()
                } else {
                    "off".dimmed().to_string()
                }
            ),
            "Save and exit".green().bold().to_string(),
        ];

        let selection = Select::new()
            .with_prompt("  Select an option to change")
            .items(&items)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                cfg.images = Confirm::new()
                    .with_prompt("  Show images in search results? (requires iTerm2/Kitty/WezTerm)")
                    .default(cfg.images)
                    .interact()?;
            }
            1 => {
                let value: u32 = Input::new()
                    .with_prompt("  Default max results (1-50)")
                    .default(cfg.max_results)
                    .validate_with(|input: &u32| {
                        if (1..=50).contains(input) {
                            Ok(())
                        } else {
                            Err("Must be between 1 and 50")
                        }
                    })
                    .interact_text()?;
                cfg.max_results = value;
            }
            2 => {
                let modes = vec!["embedding", "rerank"];
                let current = if cfg.ranking_mode == "rerank" { 1 } else { 0 };
                let choice = Select::new()
                    .with_prompt("  Default ranking mode")
                    .items(&modes)
                    .default(current)
                    .interact()?;
                cfg.ranking_mode = modes[choice].to_string();
            }
            3 => {
                cfg.include_answer = Confirm::new()
                    .with_prompt("  Include AI answer by default? (costs 2 credits per search)")
                    .default(cfg.include_answer)
                    .interact()?;
            }
            4 => break,
            _ => break,
        }

        eprintln!();
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
        if cfg.images {
            "on".green().to_string()
        } else {
            "off".to_string()
        }
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
        if cfg.include_answer {
            "on".green().to_string()
        } else {
            "off".to_string()
        }
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
            cfg.images = parse_bool(value);
        }
        "max_results" | "max-results" => {
            let n: u32 = value
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid number"))?;
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
            cfg.include_answer = parse_bool(value);
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

fn parse_bool(value: &str) -> bool {
    matches!(
        value.to_lowercase().as_str(),
        "on" | "true" | "yes" | "1"
    )
}
