use std::io;

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::Cli;

pub fn run(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "cerul", &mut io::stdout());
    Ok(())
}
