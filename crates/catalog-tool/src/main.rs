//! Reproducible catalog tooling that does not depend on Python.

mod parent;

use std::path::PathBuf;
use std::process::ExitCode;

use catalog_tool::package;

const USAGE: &str = "usage:\n  catalog-tool parent-candidates [--root <dir>] [--write | --check]\n  catalog-tool package --destination <zip path> [--root <dir>]";

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

fn next_value<'a>(
    arguments: &mut std::slice::Iter<'a, String>,
    flag: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    arguments
        .next()
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value; {USAGE}").into())
}
