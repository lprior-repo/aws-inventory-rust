//! Command line of the viewer, read once before any window exists.

use crate::worker::WorkerConfig;
use std::path::PathBuf;
use std::sync::OnceLock;

pub const USAGE: &str = "\
usage: cmdb-ui --database PATH [--graph-url URL --graph-name NAME] [--linux-backend=BACKEND]

Options:
  --database PATH          SQLite inventory database to view (required)
  --graph-url URL          FalkorDB endpoint; use with --graph-name
  --graph-name NAME        FalkorDB graph name; use with --graph-url
  --linux-backend=BACKEND  Force the Linux display backend (value must follow '=')
  --hot                    Makepad hot reload (development)
  --stdin-loop             Makepad hosted-child mode (started by a Makepad host)
  -h, --help               Print this usage and exit
";

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Run(WorkerConfig),
}

static STARTUP: OnceLock<WorkerConfig> = OnceLock::new();

/// Parses the process arguments. Nothing here touches the GUI.
pub fn from_process_args() -> Result<Command, String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    parse(&arguments)
}

/// Records the validated configuration for the app's worker. Set once, before the window.
pub fn set_startup_config(config: WorkerConfig) {
    let _ = STARTUP.set(config);
}

/// The configuration recorded at startup; defaults when none was recorded (tests).
pub fn startup_config() -> WorkerConfig {
    STARTUP.get().cloned().unwrap_or_default()
}

pub fn parse(arguments: &[String]) -> Result<Command, String> {
    if arguments
        .iter()
        .any(|argument| argument == "-h" || argument == "--help")
    {
        return Ok(Command::Help);
    }
    let mut config = WorkerConfig::default();
    parse_arguments(arguments, &mut config)?;
    Ok(Command::Run(config))
}

fn parse_arguments(arguments: &[String], config: &mut WorkerConfig) -> Result<(), String> {
    match arguments {
        [] => Ok(()),
        [flag, remaining @ ..] if is_switch(flag) => parse_arguments(remaining, config),
        [flag, value, remaining @ ..] if is_value_flag(flag) => {
            set_argument(config, flag, value)?;
            parse_arguments(remaining, config)
        }
        [flag, ..] if is_value_flag(flag) => Err(format!("missing value for {flag}")),
        [flag, ..] => Err(format!("unknown argument {flag}")),
    }
}

fn is_switch(flag: &str) -> bool {
    matches!(flag, "--hot" | "--stdin-loop") || flag.starts_with("--linux-backend=")
}

fn is_value_flag(flag: &str) -> bool {
    matches!(flag, "--database" | "--graph-url" | "--graph-name")
}

fn set_argument(config: &mut WorkerConfig, flag: &str, value: &str) -> Result<(), String> {
    match flag {
        "--database" => config.database = Some(PathBuf::from(value)),
        "--graph-url" => config.graph_url = Some(value.to_owned()),
        "--graph-name" => config.graph_name = Some(value.to_owned()),
        _ => return Err(format!("unknown argument {flag}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn given_help_flag_when_parsed_then_help_wins_anywhere() {
        assert_eq!(parse(&args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["-h"])), Ok(Command::Help));
        assert_eq!(
            parse(&args(&["--database", "a.db", "--help"])),
            Ok(Command::Help)
        );
    }

    #[test]
    fn given_no_arguments_when_parsed_then_run_with_default_config() {
        assert_eq!(parse(&[]), Ok(Command::Run(WorkerConfig::default())));
    }

    #[test]
    fn given_connection_flags_when_parsed_then_config_carries_them() {
        let command = parse(&args(&[
            "--database",
            "inventory.db",
            "--graph-url",
            "redis://127.0.0.1:6379",
            "--graph-name",
            "cmdb",
            "--hot",
            "--stdin-loop",
        ]));
        assert_eq!(
            command,
            Ok(Command::Run(WorkerConfig {
                database: Some(PathBuf::from("inventory.db")),
                graph_url: Some("redis://127.0.0.1:6379".to_owned()),
                graph_name: Some("cmdb".to_owned()),
            }))
        );
    }

    #[test]
    fn given_linux_backend_equals_value_when_parsed_then_passed_through() {
        assert_eq!(
            parse(&args(&["--linux-backend=x11", "--database", "a.db"])),
            Ok(Command::Run(WorkerConfig {
                database: Some(PathBuf::from("a.db")),
                ..WorkerConfig::default()
            }))
        );
    }

    #[test]
    fn given_space_separated_linux_backend_when_parsed_then_unknown_argument() {
        assert_eq!(
            parse(&args(&["--linux-backend", "x11"])),
            Err("unknown argument --linux-backend".to_owned())
        );
    }

    #[test]
    fn given_unknown_flag_when_parsed_then_error_names_it() {
        assert_eq!(
            parse(&args(&["--bogus"])),
            Err("unknown argument --bogus".to_owned())
        );
        assert_eq!(
            parse(&args(&["--bogus", "value"])),
            Err("unknown argument --bogus".to_owned())
        );
    }

    #[test]
    fn given_value_flag_without_value_when_parsed_then_error_names_it() {
        assert_eq!(
            parse(&args(&["--database"])),
            Err("missing value for --database".to_owned())
        );
    }

    #[test]
    fn given_usage_when_read_then_every_supported_flag_is_listed() {
        for flag in [
            "--database",
            "--graph-url",
            "--graph-name",
            "--linux-backend=",
            "--hot",
            "--stdin-loop",
            "--help",
            "-h",
        ] {
            assert!(USAGE.contains(flag), "usage lacks {flag}");
        }
    }
}
