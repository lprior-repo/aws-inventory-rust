use crate::cli::{Cli, Command};
use anyhow::Result;
use clap::Parser;
use inventory_core::catalog::Catalog;
use std::{io::Write, process::ExitCode};

const BYTES: &[u8] = include_bytes!("../catalog/aws-api.catalog");

pub(super) fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    if matches!(cli.command, Command::Catalog { models: true }) {
        let text = Catalog::open(BYTES)?.counts()?.render()?;
        std::io::stdout().lock().write_all(text.as_bytes())?;
        return Ok(ExitCode::SUCCESS);
    }
    super::execute(cli.command).map(super::output)
}
