//! Tests for the serve command CLI parsing. Interval parsing tests live in
//! src/serve_command/config.rs to avoid duplication.

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
fn serve_cli_interval_value() {
    let cli = crate::cli::Cli::try_parse_from([
        "aws-inventory",
        "serve",
        "--database",
        "test.sqlite",
        "--profile",
        "audit",
        "--interval",
        "15m",
    ]);
    let args = match cli.unwrap().command {
        crate::cli::Command::Serve(args) => args,
        other => panic!("expected Serve command, got {other:?}"),
    };
    assert_eq!(args.interval, "15m");
}

#[test]
fn serve_cli_default_interval() {
    let cli =
        crate::cli::Cli::try_parse_from(["aws-inventory", "serve", "--database", "test.sqlite"]);
    let args = match cli.unwrap().command {
        crate::cli::Command::Serve(args) => args,
        other => panic!("expected Serve command, got {other:?}"),
    };
    assert_eq!(args.interval, "15m");
}

#[test]
fn serve_cli_requires_database() {
    let result = crate::cli::Cli::try_parse_from(["aws-inventory", "serve"]);
    assert!(result.is_err(), "expected parse error without --database");
}
