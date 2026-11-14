//! Reproducible catalog tooling that does not depend on Python.

mod parent;

use std::path::PathBuf;
use std::process::ExitCode;

use catalog_tool::{package, smithy};

const USAGE: &str = "usage:\n  catalog-tool parent-candidates [--root <dir>] [--write | --check]\n  catalog-tool package --destination <zip path> [--root <dir>]\n  catalog-tool models report [--source <dir>] [--root <dir>] [--check | --write]";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = arguments.iter();
    match arguments.next().map(String::as_str) {
        Some("parent-candidates") => parent_arguments(&mut arguments),
        Some("package") => package_arguments(&mut arguments),
        Some("models") => models_arguments(&mut arguments),
        _ => Err(USAGE.into()),
    }
}

fn parent_arguments(
    arguments: &mut std::slice::Iter<'_, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root = default_root();
    let mut write = false;
    let mut check = false;
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--root" => root = PathBuf::from(next_value(arguments, "--root")?),
            "--write" => write = true,
            "--check" => check = true,
            other => return Err(format!("unexpected {other}; {USAGE}").into()),
        }
    }
    parent::run(&root, write, check)
}

fn package_arguments(
    arguments: &mut std::slice::Iter<'_, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root = default_root();
    let mut destination = None;
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--root" => root = PathBuf::from(next_value(arguments, "--root")?),
            "--destination" | "--output" => {
                destination = Some(PathBuf::from(next_value(arguments, "--destination")?));
            }
            other => return Err(format!("unexpected {other}; {USAGE}").into()),
        }
    }
    let destination = destination.ok_or_else(|| format!("--destination is required; {USAGE}"))?;
    package::run(&root, &destination)
}

fn default_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn models_arguments(
    arguments: &mut std::slice::Iter<'_, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let subcommand = arguments
        .next()
        .map(String::as_str)
        .ok_or_else(|| format!("expected 'report' subcommand; {USAGE}"))?;
    if subcommand != "report" {
        return Err(format!("unexpected '{subcommand}'; {USAGE}").into());
    }
    let flags = parse_models_flags(arguments)?;
    let root = flags.root.map_or_else(default_root, PathBuf::from);
    let source = flags
        .source
        .map_or_else(|| root.join(".local/aws-models"), PathBuf::from);
    let mode = flags.mode.unwrap_or(smithy::ReportMode::Print);
    Ok(smithy::run(&root, &source, mode)?)
}

#[derive(Default)]
struct ModelsFlags<'a> {
    root: Option<&'a str>,
    source: Option<&'a str>,
    mode: Option<smithy::ReportMode>,
    pending: Option<&'static str>,
}

fn parse_models_flags<'a>(
    arguments: &mut std::slice::Iter<'a, String>,
) -> Result<ModelsFlags<'a>, Box<dyn std::error::Error>> {
    let flags = arguments.try_fold(ModelsFlags::default(), |flags, arg| {
        parse_models_flag(flags, arg)
    })?;
    if let Some(flag) = flags.pending {
        return Err(format!("{flag} requires a value; {USAGE}").into());
    }
    Ok(flags)
}

fn parse_models_flag<'a>(
    mut flags: ModelsFlags<'a>,
    value: &'a str,
) -> Result<ModelsFlags<'a>, Box<dyn std::error::Error>> {
    if let Some(flag) = flags.pending.take() {
        if value.starts_with("--") {
            return Err(format!("{flag} requires a value; {USAGE}").into());
        }
        match flag {
            "--root" => flags.root = Some(value),
            _ => flags.source = Some(value),
        }
    } else {
        apply_models_flag(&mut flags, value)?;
    }
    Ok(flags)
}

fn apply_models_flag(
    flags: &mut ModelsFlags<'_>,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        "--root" if flags.root.is_none() => flags.pending = Some("--root"),
        "--source" if flags.source.is_none() => flags.pending = Some("--source"),
        "--check" | "--write" if flags.mode.is_none() => {
            flags.mode = Some(if value == "--check" {
                smithy::ReportMode::Check
            } else {
                smithy::ReportMode::Write
            });
        }
        "--root" | "--source" | "--check" | "--write" => {
            return Err(format!("repeated or conflicting flag {value}; {USAGE}").into());
        }
        _ => return Err(format!("unexpected '{value}'; {USAGE}").into()),
    }
    Ok(())
}

fn next_value<'a>(
    arguments: &mut std::slice::Iter<'a, String>,
    flag: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    arguments
        .next()
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value; {USAGE}").into())
}
