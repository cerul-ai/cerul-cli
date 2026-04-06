use std::io::IsTerminal;

use anyhow::{bail, Result};
use colored::Colorize;

use crate::{
    client::CerulClient,
    config::{self, read_line},
    output,
    types::{RankingMode, SearchFilters, SearchRequest},
    SearchArgs,
};

pub async fn run(client: &CerulClient, args: SearchArgs) -> Result<()> {
    let cfg = config::load_config();
    let max_results = args.max_results.unwrap_or(cfg.max_results);
    let ranking_mode = args.ranking_mode.unwrap_or_else(|| match cfg.ranking_mode.as_str() {
        "rerank" => RankingMode::Rerank,
        _ => RankingMode::Embedding,
    });
    let include_answer = args.include_answer || cfg.include_answer;
    let is_json = args.json;
    let is_agent = args.agent;
    let is_interactive = !is_json && !is_agent && std::io::stderr().is_terminal();

    // First search with the provided query
    let mut query = args.query.clone();
    let filters = build_filters(&args);

    loop {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            bail!("query must contain at least one non-whitespace character");
        }
        if trimmed.chars().count() > 400 {
            bail!("query must be 400 characters or fewer");
        }

        let request = SearchRequest {
            query: trimmed.to_string(),
            max_results,
            ranking_mode,
            include_answer,
            filters: filters.clone(),
        };

        let response = client.search(&request).await?;

        // Save to history
        history::append(trimmed, response.results.len());

        if is_json {
            output::print_json(&response)?;
            return Ok(());
        }

        if is_agent {
            output::print_search_agent(&response);
            return Ok(());
        }

        output::print_search_human(&response, cfg.images);

        // Interactive: prompt for another search
        if !is_interactive {
            return Ok(());
        }

        let next = read_line(&format!(
            "  {} ",
            "Search again (or Enter to exit):".dimmed()
        ))?;

        if next.is_empty() {
            return Ok(());
        }

        query = next;
        eprintln!();
    }
}

fn build_filters(args: &SearchArgs) -> Option<SearchFilters> {
    let filters = SearchFilters {
        speaker: args.speaker.clone(),
        published_after: args.published_after.clone(),
        min_duration: args.min_duration,
        max_duration: args.max_duration,
        source: args.source.clone(),
    };

    if filters.speaker.is_none()
        && filters.published_after.is_none()
        && filters.min_duration.is_none()
        && filters.max_duration.is_none()
        && filters.source.is_none()
    {
        None
    } else {
        Some(filters)
    }
}

fn is_valid_date(value: &str) -> bool {
    if value.len() != 10 {
        return false;
    }
    let bytes = value.as_bytes();
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

// ── Search history ──────────────────────────────────────────────────

pub mod history {
    use std::{fs, path::PathBuf};

    use crate::config;

    struct HistoryEntry {
        timestamp: String,
        query: String,
        result_count: usize,
    }

    fn history_path() -> Option<PathBuf> {
        config::config_file_path()
            .ok()
            .map(|p| p.with_file_name("history.tsv"))
    }

    pub fn append(query: &str, result_count: usize) {
        let Some(path) = history_path() else {
            return;
        };
        let timestamp = now_iso();
        let line = format!("{}\t{}\t{}\n", timestamp, result_count, query);
        // Append to file, create if not exists
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = file.write_all(line.as_bytes());
        }
    }

    pub fn read_recent(limit: usize) -> Vec<(String, String, usize)> {
        let Some(path) = history_path() else {
            return vec![];
        };
        let content = fs::read_to_string(&path).unwrap_or_default();
        let mut entries: Vec<(String, String, usize)> = content
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '\t').collect();
                if parts.len() == 3 {
                    Some((
                        parts[0].to_string(),
                        parts[2].to_string(),
                        parts[1].parse().unwrap_or(0),
                    ))
                } else {
                    None
                }
            })
            .collect();
        entries.reverse();
        entries.truncate(limit);
        entries
    }

    fn now_iso() -> String {
        // Simple UTC timestamp without chrono dependency
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Convert to rough ISO format
        let days = secs / 86400;
        let time_secs = secs % 86400;
        let hours = time_secs / 3600;
        let minutes = (time_secs % 3600) / 60;
        // Approximate date calculation (good enough for display)
        let years = 1970 + days / 365;
        let remaining_days = days % 365;
        let month = remaining_days / 30 + 1;
        let day = remaining_days % 30 + 1;
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            years,
            month.min(12),
            day.min(31),
            hours,
            minutes
        )
    }
}
