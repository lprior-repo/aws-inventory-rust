//! Command line entry point for the SDK operation inventory.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sdk_operations::Error;
use sdk_operations::inventory::{build, render, verify, write};

const USAGE: &str = "usage: sdk-operations inventory --registry <dir> --out <file> [--check] \
[--lockfile <file>]";

struct Options {
    registry: PathBuf,
    out: PathBuf,
    lockfile: PathBuf,
    check: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sdk-operations: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Error> {
    let options = parse(std::env::args_os().skip(1).collect())?;
    let inventory = build(&options.registry, &options.lockfile)?;
    let bytes = render(&inventory)?;
    if options.check {
        verify(&options.out, &bytes)
    } else {
        write(&options.out, &bytes)
    }
}

fn parse(arguments: Vec<OsString>) -> Result<Options, Error> {
    let mut iterator = arguments.into_iter();
    match iterator.next().as_deref() {
        Some(command) if command == "inventory" => parse_inventory(iterator),
        _ => Err(Error::Usage(USAGE.to_owned())),
    }
}

fn parse_inventory(mut iterator: impl Iterator<Item = OsString>) -> Result<Options, Error> {
    let mut registry = None;
    let mut out = None;
    let mut lockfile = None;
    let mut check = false;
    while let Some(argument) = iterator.next() {
        match argument.to_str() {
            Some("--check") => check = true,
            Some("--registry") => registry = Some(path_value(&mut iterator, "--registry")?),
            Some("--out") => out = Some(path_value(&mut iterator, "--out")?),
            Some("--lockfile") => lockfile = Some(path_value(&mut iterator, "--lockfile")?),
            _ => return Err(Error::Usage(USAGE.to_owned())),
        }
    }
    Ok(Options {
        registry: registry.ok_or_else(usage)?,
        out: out.ok_or_else(usage)?,
        lockfile: match lockfile {
            Some(path) => path,
            None => workspace_lockfile(),
        },
        check,
    })
}

fn path_value(iterator: &mut impl Iterator<Item = OsString>, flag: &str) -> Result<PathBuf, Error> {
    iterator
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| Error::Usage(format!("{flag} requires a path\n{USAGE}")))
}

fn usage() -> Error {
    Error::Usage(USAGE.to_owned())
}

fn workspace_lockfile() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("Cargo.lock")
}
