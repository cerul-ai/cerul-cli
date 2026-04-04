use anyhow::Result;
use colored::Colorize;
use dialoguer::Select;

use crate::client::CerulClient;
use crate::config;
use crate::output;

use super::search::history;

pub fn run(limit: usize) -> Result<()> {
    let entries = history::read_recent(limit);

    if entries.is_empty() {
        eprintln!();
        eprintln!(
            "  No search history yet. Try: {}",
            "cerul search \"sam altman agi\"".green()
        );
        eprintln!();
        return Ok(());
    }

    eprintln!();
    eprintln!("  {}", "📋 Recent Searches".bold());
    eprintln!("  {}", "Select to search again, Esc to exit".dimmed());
    eprintln!();

    let display_items: Vec<String> = entries
        .iter()
        .map(|(timestamp, query, result_count)| {
            format!(
                "{}  {}  {}",
                timestamp.dimmed(),
                query,
                format!("({result_count} results)").dimmed(),
            )
        })
        .collect();

    let selection = Select::new()
        .items(&display_items)
        .default(0)
        .with_prompt("  ")
        .interact_opt()?;

    let Some(index) = selection else {
        return Ok(());
    };

    let (_, query, _) = &entries[index];
    eprintln!();
    eprintln!("  Re-searching: {}", query.bold());
    eprintln!();

    // Re-run the search
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        let client = CerulClient::from_config()?;
        let cfg = config::load_config();

        let ranking_mode = match cfg.ranking_mode.as_str() {
            "rerank" => crate::types::RankingMode::Rerank,
            _ => crate::types::RankingMode::Embedding,
        };

        let request = crate::types::SearchRequest {
            query: query.clone(),
            max_results: cfg.max_results,
            ranking_mode,
            include_answer: cfg.include_answer,
            filters: None,
        };

        let response = client.search(&request).await?;
        history::append(query, response.results.len());
        output::print_search_human(&response, cfg.images);
        Ok(())
    })
}
