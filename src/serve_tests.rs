use clap::Parser;

const DATABASE: &str = "postgresql://inventory@localhost:5432/inventory";

fn serve_args(extra: &[&str]) -> anyhow::Result<crate::cli::ServeArgs> {
    let mut argv = vec!["aws-inventory", "serve", "--database", DATABASE];
    argv.extend_from_slice(extra);
    let cli = crate::cli::Cli::try_parse_from(argv)?;
    match cli.command {
        crate::cli::Command::Serve(args) => Ok(args),
        other => anyhow::bail!("expected Serve command, got {other:?}"),
    }
}

#[test]
fn serve_cli_parses_required_args() {
    let cli = crate::cli::Cli::try_parse_from([
        "aws-inventory",
        "serve",
        "--database",
        DATABASE,
        "--profile",
        "audit",
    ]);
    assert!(cli.is_ok(), "parse failed: {:?}", cli);
}

#[test]
fn serve_cli_interval_value() -> anyhow::Result<()> {
    let args = serve_args(&["--profile", "audit", "--interval", "15m"])?;
    assert_eq!(args.interval, "15m");
    Ok(())
}

#[test]
fn serve_cli_default_interval() -> anyhow::Result<()> {
    let args = serve_args(&[])?;
    assert_eq!(args.interval, "15m");
    Ok(())
}

#[test]
fn serve_cli_requires_database() {
    let result = crate::cli::Cli::try_parse_from(["aws-inventory", "serve"]);
    assert!(result.is_err(), "expected parse error without --database");
}

#[test]
fn serve_cli_debug_output_redacts_the_database_url() -> anyhow::Result<()> {
    let args = serve_args(&[])?;
    assert_eq!(args.database, DATABASE);
    let rendered = format!("{args:?}");
    assert!(!rendered.contains(DATABASE), "Debug leaked: {rendered}");
    assert!(rendered.contains("<redacted>"));
    Ok(())
}

#[cfg(feature = "sweep")]
#[tokio::test]
async fn given_zero_idle_timeout_when_serve_starts_then_it_fails_before_opening_the_store()
-> anyhow::Result<()> {
    let args = serve_args(&[
        "--catalog",
        "missing.catalog",
        "--idle-timeout-ms",
        "0",
        "--pool-idle-per-host",
        "1",
    ])?;

    let result = crate::serve_command::run(args).await;
    assert_eq!(
        result.err().map(|error| error.to_string()).as_deref(),
        Some(
            "invalid sweep configuration: --idle-timeout-ms (INVENTORY_SWEEP_IDLE_TIMEOUT_MS) \
             was 0; allowed range is 1..=90000 ms"
        )
    );
    Ok(())
}
