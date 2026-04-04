use anyhow::Result;

use crate::{client::CerulClient, output, UsageArgs};

pub async fn run(client: &CerulClient, args: UsageArgs) -> Result<()> {
    let response = client.usage().await?;

    if args.json {
        output::print_json(&response)?;
    } else {
        output::print_usage_human(&response);
    }

    Ok(())
}
