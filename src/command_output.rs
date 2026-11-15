use crate::{cli::Command, dispatch};
use anyhow::Result;
use serde_json::Value;
use std::{io::Write, process::ExitCode};

pub(super) enum CommandOutput {
    Legacy(Value),
    Native(ExitCode),
}

pub(super) fn execute(command: Command) -> Result<CommandOutput> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .try_init()
        .map_err(|error| anyhow::anyhow!("logging initialization: {error}"))?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(dispatch(command))
}

pub(super) fn output(result: CommandOutput) -> ExitCode {
    let value = match result {
        CommandOutput::Legacy(value) => value,
        CommandOutput::Native(code) => return code,
    };
    let mut stdout = std::io::stdout().lock();
    let result = serde_json::to_writer_pretty(&mut stdout, &value)
        .map_err(std::io::Error::other)
        .and_then(|()| stdout.write_all(b"\n"));
    if let Err(error) = result {
        eprintln!("output error: {error}");
        return ExitCode::FAILURE;
    }
    match value.get("status").and_then(Value::as_str) {
        Some(status @ ("partial" | "cancelled")) => {
            eprintln!("scan status is {status}: the report on standard output lists what failed");
            ExitCode::from(2)
        }
        _ => ExitCode::SUCCESS,
    }
}
