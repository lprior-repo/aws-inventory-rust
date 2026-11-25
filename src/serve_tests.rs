use clap::Parser;

#[test]
fn serve_cli_parses_required_args() {
    let cli = crate::cli::Cli::try_parse_from([
        "aws-inventory",
        "serve",
        "--database",
        "test.sqlite",
        "--profile",
        "audit",
    ]);
    assert!(cli.is_ok(), "parse failed: {:?}", cli);
}

#[test]
fn serve_cli_interval_value() -> anyhow::Result<()> {
    let cli = crate::cli::Cli::try_parse_from([
        "aws-inventory",
        "serve",
        "--database",
        "test.sqlite",
        "--profile",
        "audit",
        "--interval",
        "15m",
    ])?;
    let args = match cli.command {
        crate::cli::Command::Serve(args) => args,
        other => anyhow::bail!("expected Serve command, got {other:?}"),
    };
    assert_eq!(args.interval, "15m");
    Ok(())
}

#[test]
fn serve_cli_default_interval() -> anyhow::Result<()> {
    let cli =
        crate::cli::Cli::try_parse_from(["aws-inventory", "serve", "--database", "test.sqlite"])?;
    let args = match cli.command {
        crate::cli::Command::Serve(args) => args,
        other => anyhow::bail!("expected Serve command, got {other:?}"),
    };
    assert_eq!(args.interval, "15m");
    Ok(())
}

#[test]
fn serve_cli_requires_database() {
    let result = crate::cli::Cli::try_parse_from(["aws-inventory", "serve"]);
    assert!(result.is_err(), "expected parse error without --database");
}

#[cfg(feature = "aws")]
#[tokio::test]
async fn given_zero_idle_timeout_when_serve_starts_then_it_fails_before_opening_the_store()
-> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.bin");
    let catalog = directory.path().join("missing.catalog");
    let cli = crate::cli::Cli::try_parse_from([
        std::ffi::OsStr::new("aws-inventory"),
        std::ffi::OsStr::new("serve"),
        std::ffi::OsStr::new("--database"),
        database.as_os_str(),
        std::ffi::OsStr::new("--catalog"),
        catalog.as_os_str(),
        std::ffi::OsStr::new("--idle-timeout-ms"),
        std::ffi::OsStr::new("0"),
        std::ffi::OsStr::new("--pool-idle-per-host"),
        std::ffi::OsStr::new("1"),
    ])?;
    let crate::cli::Command::Serve(args) = cli.command else {
        anyhow::bail!("expected Serve command");
    };

    let result = crate::serve_command::run(args).await;
    assert_eq!(
        result.err().map(|error| error.to_string()).as_deref(),
        Some(
            "invalid sweep configuration: --idle-timeout-ms (INVENTORY_SWEEP_IDLE_TIMEOUT_MS) \
             was 0; allowed range is 1..=90000 ms"
        )
    );
    assert!(!database.exists());
    Ok(())
}
