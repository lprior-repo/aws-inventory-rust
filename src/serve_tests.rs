//! Tests for the serve command: CLI parsing, interval parsing, and
//! last-completed scheduling behavior.

use crate::serve_command::parse_interval;
use clap::{CommandFactory, Parser};
use std::time::Duration;

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
    assert!(
        cli.is_ok(),
        "parse failed: {:?}",
        cli
    );
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
    let cli = crate::cli::Cli::try_parse_from([
        "aws-inventory",
        "serve",
        "--database",
        "test.sqlite",
    ]);
    let args = match cli.unwrap().command {
        crate::cli::Command::Serve(args) => args,
        other => panic!("expected Serve command, got {other:?}"),
    };
    assert_eq!(args.interval, "15m");
}

#[test]
fn serve_cli_requires_database() {
    let result = crate::cli::Cli::try_parse_from(["aws-inventory", "serve"]);
    assert!(
        result.is_err(),
        "expected parse error without --db"
    );
}

#[test]
fn parse_interval_seconds() {
    let d = parse_interval("30s").expect("parse 30s");
    assert_eq!(d, Duration::from_secs(30));
}

#[test]
fn parse_interval_minutes() {
    let d = parse_interval("5m").expect("parse 5m");
    assert_eq!(d, Duration::from_secs(300));
}

#[test]
fn parse_interval_hours() {
    let d = parse_interval("2h").expect("parse 2h");
    assert_eq!(d, Duration::from_secs(7200));
}

#[test]
fn parse_interval_days() {
    let d = parse_interval("1d").expect("parse 1d");
    assert_eq!(d, Duration::from_secs(86400));
}

#[test]
fn parse_interval_zero_rejected() {
    let result = parse_interval("0m");
    assert!(
        result.is_err(),
        "expected error for zero interval"
    );
}

#[test]
fn parse_interval_unknown_unit_rejected() {
    let result = parse_interval("5w");
    assert!(
        result.is_err(),
        "expected error for unknown unit 'w'"
    );
}

#[test]
fn parse_interval_no_unit_rejected() {
    let result = parse_interval("5");
    assert!(
        result.is_err(),
        "expected error for missing unit"
    );
}