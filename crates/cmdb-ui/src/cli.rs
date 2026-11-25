use crate::worker::{WorkerConfig, parse_lod_config};
use std::sync::OnceLock;

pub const USAGE: &str = "\
usage: cmdb-ui --database TARGET [--frame-harness SECONDS] [--graph-url URL --graph-name NAME] [--linux-backend=BACKEND]

Options:
  --database TARGET        SQLite path or PostgreSQL inventory URL (required)
  --frame-harness SECONDS  Scripted frame capture (1..600 seconds); reports and exits
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

/// Parses the process arguments and `CMDB_UI_LOD`. Nothing here touches the GUI.
pub fn from_process_args() -> Result<Command, String> {
    let arguments: Vec<String> = std::env::args().skip(1).take(65).collect();
    let lod = std::env::var("CMDB_UI_LOD").ok();
    with_lod(parse(&arguments)?, lod.as_deref())
}

/// Applies a level-of-detail override; an invalid value is a startup error, never a silent default.
pub fn with_lod(command: Command, lod: Option<&str>) -> Result<Command, String> {
    match (command, lod) {
        (Command::Run(config), Some(value)) => Ok(Command::Run(WorkerConfig {
            graph_limits: parse_lod_config(value)?,
            ..config
        })),
        (command, _) => Ok(command),
    }
}

/// Records the validated configuration for the app's worker. Set once, before the window.
pub fn set_startup_config(config: WorkerConfig) {
    let _ = STARTUP.set(config);
}

/// The configuration recorded at startup; defaults when none was recorded (tests).
pub fn startup_config() -> WorkerConfig {
    match STARTUP.get() {
        Some(config) => config.clone(),
        None => WorkerConfig::default(),
    }
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
    if config.frame_harness.is_some() && config.database.is_none() {
        return Err("--frame-harness requires --database PATH".to_owned());
    }
    Ok(Command::Run(config))
}

fn parse_arguments(arguments: &[String], config: &mut WorkerConfig) -> Result<(), String> {
    if arguments.len() > 64 {
        return Err("at most 64 command-line arguments are accepted".to_owned());
    }
    let mut arguments = arguments.iter();
    for _ in 0..64 {
        let Some(flag) = arguments.next() else { break };
        if is_switch(flag) {
            continue;
        }
        if !is_value_flag(flag) {
            return Err(unknown_argument(flag));
        }
        let value = arguments
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        set_argument(config, flag, value)?;
    }
    Ok(())
}

fn is_switch(flag: &str) -> bool {
    matches!(flag, "--hot" | "--stdin-loop") || flag.starts_with("--linux-backend=")
}

fn is_value_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--database" | "--graph-url" | "--graph-name" | "--frame-harness"
    )
}

fn set_argument(config: &mut WorkerConfig, flag: &str, value: &str) -> Result<(), String> {
    match flag {
        "--database" => config.database = Some(value.to_owned()),
        "--graph-url" => config.graph_url = Some(value.to_owned()),
        "--graph-name" => config.graph_name = Some(value.to_owned()),
        "--frame-harness" => {
            config.frame_harness = Some(crate::frame_harness::FrameConfig::parse(value)?)
        }
        _ => return Err(unknown_argument(flag)),
    }
    Ok(())
}

fn unknown_argument(flag: &str) -> String {
    let contains_postgres_url = flag.match_indices("://").any(|(scheme_end, _)| {
        [8, 10].into_iter().any(|scheme_len| {
            scheme_end
                .checked_sub(scheme_len)
                .and_then(|start| flag.get(start..))
                .is_some_and(cmdb_view::is_postgres_url)
        })
    });
    if contains_postgres_url {
        "unknown argument [PostgreSQL URL redacted]".to_owned()
    } else {
        format!("unknown argument {flag}")
    }
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
                database: Some("inventory.db".to_owned()),
                graph_url: Some("redis://127.0.0.1:6379".to_owned()),
                graph_name: Some("cmdb".to_owned()),
                ..WorkerConfig::default()
            }))
        );
    }

    #[test]
    fn given_linux_backend_equals_value_when_parsed_then_passed_through() {
        assert_eq!(
            parse(&args(&["--linux-backend=x11", "--database", "a.db"])),
            Ok(Command::Run(WorkerConfig {
                database: Some("a.db".to_owned()),
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

#[cfg(test)]
mod lod_tests {
    use super::*;

    #[test]
    fn given_invalid_lod_when_applied_then_startup_fails() {
        let run = Command::Run(WorkerConfig::default());
        assert!(with_lod(run, Some("2:abc:5")).is_err());
    }

    #[test]
    fn given_valid_lod_when_applied_then_limits_replace_defaults()
    -> Result<(), Box<dyn std::error::Error>> {
        let run = Command::Run(WorkerConfig::default());
        let Command::Run(config) = with_lod(run, Some("3:10:20"))? else {
            return Err("valid lod must yield a run command".into());
        };
        assert_eq!(
            (
                config.graph_limits.depth,
                config.graph_limits.nodes,
                config.graph_limits.edges
            ),
            (3, 10, 20)
        );
        Ok(())
    }

    #[test]
    fn given_help_when_lod_is_invalid_then_help_still_wins() {
        assert_eq!(with_lod(Command::Help, Some("bad")), Ok(Command::Help));
    }
}

#[cfg(test)]
mod frame_harness_tests;

#[cfg(test)]
mod postgres_tests {
    use super::*;

    #[test]
    fn postgres_given_url_when_parsing_database_then_target_is_retained_and_debug_is_redacted()
    -> Result<(), String> {
        let target = "postgres://reader:secret-password@localhost/inventory";
        let Command::Run(config) = parse(&["--database".to_owned(), target.to_owned()])? else {
            return Err("expected run command".into());
        };
        assert_eq!(config.database.as_deref(), Some(target));
        assert!(!format!("{config:?}").contains("secret-password"));
        Ok(())
    }

    #[test]
    fn postgres_given_url_in_unknown_argument_when_parsing_then_password_is_not_echoed()
    -> Result<(), String> {
        for flag in [
            "postgres://reader:secret-password@localhost/inventory",
            "--database=postgresql://reader:secret-password@localhost/inventory",
            "POSTGRES://reader:secret-password@localhost/inventory",
            "--database=PoStGrEsQl://reader:secret-password@localhost/inventory",
            "-postgres://reader:secret-password@localhost/inventory",
            "--dbpostgres://reader:secret-password@localhost/inventory",
        ] {
            let error = parse(&[flag.to_owned()])
                .err()
                .ok_or("expected unknown argument")?;
            assert_eq!(error, "unknown argument [PostgreSQL URL redacted]");
        }
        Ok(())
    }
}
