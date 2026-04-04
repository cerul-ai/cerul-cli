use std::io::{self, Write};

use anyhow::{bail, Result};
use colored::Colorize;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal,
};

use crate::config::{self, UserConfig};

struct ConfigField {
    label: &'static str,
    options: Vec<String>,
    selected: usize,
}

impl ConfigField {
    fn bool_field(label: &'static str, value: bool) -> Self {
        Self {
            label,
            options: vec!["off".into(), "on".into()],
            selected: if value { 1 } else { 0 },
        }
    }

    fn enum_field(label: &'static str, options: Vec<String>, current: &str) -> Self {
        let selected = options.iter().position(|o| o == current).unwrap_or(0);
        Self {
            label,
            options,
            selected,
        }
    }

    fn value(&self) -> &str {
        &self.options[self.selected]
    }

    fn move_left(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.selected + 1 < self.options.len() {
            self.selected += 1;
        }
    }
}

pub fn run_interactive() -> Result<()> {
    let cfg = config::load_config();

    let max_results_options: Vec<String> = vec![
        "1", "3", "5", "10", "15", "20", "30", "50",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let mut fields = vec![
        ConfigField::bool_field("Images in results", cfg.images),
        ConfigField::enum_field(
            "Default max results",
            max_results_options,
            &cfg.max_results.to_string(),
        ),
        ConfigField::enum_field(
            "Default ranking mode",
            vec!["embedding".into(), "rerank".into()],
            &cfg.ranking_mode,
        ),
        ConfigField::bool_field("Include AI answer", cfg.include_answer),
    ];

    let mut cursor_row: usize = 0;

    terminal::enable_raw_mode()?;
    let result = run_config_loop(&mut fields, &mut cursor_row);
    terminal::disable_raw_mode()?;

    if let Err(e) = result {
        return Err(e);
    }

    // Build config from selections
    let new_cfg = UserConfig {
        images: fields[0].value() == "on",
        max_results: fields[1].value().parse().unwrap_or(5),
        ranking_mode: fields[2].value().to_string(),
        include_answer: fields[3].value() == "on",
    };

    config::save_config(&new_cfg)?;

    let path = config::config_file_path()?;
    eprintln!(
        "\r  {} Saved to {}",
        "✅".green(),
        path.display().to_string().dimmed()
    );

    // Warn if images enabled but terminal doesn't support it
    if new_cfg.images {
        let term = std::env::var("TERM_PROGRAM").unwrap_or_default();
        let supported = matches!(term.as_str(), "iTerm.app" | "WezTerm" | "kitty");
        if !supported {
            eprintln!();
            eprintln!(
                "  {} Images enabled but your terminal ({}) may not support inline images.",
                "⚠️".yellow(),
                if term.is_empty() {
                    "unknown".to_string()
                } else {
                    term
                }
            );
            eprintln!("  Supported: iTerm2, WezTerm, Kitty. Images will be skipped in other terminals.");
        }
    }

    eprintln!();

    Ok(())
}

fn run_config_loop(fields: &mut [ConfigField], cursor_row: &mut usize) -> Result<()> {
    let mut stderr = io::stderr();

    // Initial draw
    draw_config(&mut stderr, fields, *cursor_row, false)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Up => {
                    if *cursor_row > 0 {
                        *cursor_row -= 1;
                    }
                }
                KeyCode::Down => {
                    if *cursor_row < fields.len() {
                        // fields.len() = Save row
                        *cursor_row += 1;
                    }
                }
                KeyCode::Left => {
                    if *cursor_row < fields.len() {
                        fields[*cursor_row].move_left();
                    }
                }
                KeyCode::Right => {
                    if *cursor_row < fields.len() {
                        fields[*cursor_row].move_right();
                    }
                }
                KeyCode::Enter => {
                    if *cursor_row == fields.len() {
                        // Save row selected
                        draw_config(&mut stderr, fields, *cursor_row, true)?;
                        return Ok(());
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    draw_config(&mut stderr, fields, *cursor_row, true)?;
                    return Ok(());
                }
                _ => {}
            }

            draw_config(&mut stderr, fields, *cursor_row, false)?;
        }
    }
}

fn draw_config(
    w: &mut impl Write,
    fields: &[ConfigField],
    cursor_row: usize,
    final_draw: bool,
) -> Result<()> {
    // Move to start and clear
    let total_lines = fields.len() + 6; // header + hint + blank + fields + blank + save
    write!(w, "\r")?;
    // Move up to redraw (skip on first draw — we use a static offset)
    for _ in 0..total_lines {
        write!(w, "{}", cursor::MoveUp(1))?;
        write!(w, "{}", terminal::Clear(terminal::ClearType::CurrentLine))?;
    }

    write!(w, "\r\n")?;
    write!(w, "  {}\r\n", "⚙️  Cerul Configuration".bold())?;
    write!(
        w,
        "  {}\r\n",
        "↑↓ navigate  ←→ change  Enter save  Esc quit".dimmed()
    )?;
    write!(w, "\r\n")?;

    for (i, field) in fields.iter().enumerate() {
        let is_selected = i == cursor_row && !final_draw;
        let arrow = if is_selected { ">" } else { " " };

        let value_display = if is_selected {
            format!(
                "◀ {} ▶",
                field.value().bold()
            )
        } else {
            let v = field.value();
            if v == "on" {
                v.green().to_string()
            } else {
                v.to_string()
            }
        };

        write!(
            w,
            "  {} {:<24}{}\r\n",
            if is_selected {
                arrow.green().bold().to_string()
            } else {
                arrow.to_string()
            },
            field.label,
            value_display,
        )?;
    }

    // Save row
    write!(w, "\r\n")?;
    let save_selected = cursor_row == fields.len() && !final_draw;
    if save_selected {
        write!(
            w,
            "  {} {}\r\n",
            ">".green().bold(),
            "Save and exit".green().bold()
        )?;
    } else {
        write!(w, "    {}\r\n", "Save and exit".dimmed())?;
    }

    w.flush()?;
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
