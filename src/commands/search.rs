use anyhow::{bail, Result};

use crate::{
    client::CerulClient,
    output,
    types::{SearchFilters, SearchRequest},
    SearchArgs,
};

pub async fn run(client: &CerulClient, args: SearchArgs) -> Result<()> {
    let query = args.query.trim();
    if query.is_empty() {
        bail!("query must contain at least one non-whitespace character");
    }
    if query.chars().count() > 400 {
        bail!("query must be 400 characters or fewer");
    }

    if let Some(published_after) = args.published_after.as_deref() {
        if !is_valid_date(published_after) {
            bail!("published_after must be in YYYY-MM-DD format");
        }
    }

    if let (Some(min_duration), Some(max_duration)) = (args.min_duration, args.max_duration) {
        if min_duration > max_duration {
            bail!("min_duration must be less than or equal to max_duration");
        }
    }

    let filters = build_filters(&args);
    let request = SearchRequest {
        query: query.to_string(),
        max_results: args.max_results,
        ranking_mode: args.ranking_mode,
        include_answer: args.include_answer,
        filters,
    };

    let response = client.search(&request).await?;

    if args.json {
        output::print_json(&response)?;
    } else {
        output::print_search_human(&response);
    }

    Ok(())
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
