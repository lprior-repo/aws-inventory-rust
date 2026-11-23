use catalog_tool::{ReportMode, overlay};
use std::{collections::BTreeSet, path::PathBuf};

type Failure = Box<dyn std::error::Error>;

pub(super) fn overlay(arguments: &mut std::slice::Iter<'_, String>) -> Result<(), Failure> {
    let inputs = super::report_inputs(arguments)?;
    Ok(overlay::run(&inputs.root, &inputs.source, inputs.mode)?)
}

pub(super) fn overlays(arguments: &mut std::slice::Iter<'_, String>) -> Result<(), Failure> {
    if arguments.next().map(String::as_str) != Some("generate") {
        return Err("overlays requires the generate subcommand".into());
    }
    let mut flags = Flags::default();
    while let Some(flag) = arguments.next() {
        flags.apply(flag, arguments)?;
    }
    if flags.mode == Some(ReportMode::Check) && !flags.services.is_empty() {
        return Err("--service cannot be combined with --check".into());
    }
    flags.run()
}

#[derive(Default)]
struct Flags {
    root: Option<PathBuf>,
    source: Option<PathBuf>,
    services: BTreeSet<String>,
    mode: Option<ReportMode>,
}

impl Flags {
    fn run(self) -> Result<(), Failure> {
        let root = match self.root {
            Some(root) => root,
            None => super::default_root(),
        };
        let source = match self.source {
            Some(source) => source,
            None => root.join(".local/aws-models"),
        };
        let mode = match self.mode {
            Some(mode) => mode,
            None => ReportMode::Print,
        };
        overlay::generate::run(&root, &source, &self.services, mode)
    }

    fn apply(&mut self, flag: &str, arguments: &mut std::slice::Iter<'_, String>) -> Result<(), Failure> {
        match flag {
            "--root" | "--source" => self.path(flag, value(arguments, flag)?),
            "--service" => self.service(value(arguments, flag)?),
            "--write" | "--check" if self.mode.is_none() => {
                self.mode = Some(if flag == "--write" { ReportMode::Write } else { ReportMode::Check });
                Ok(())
            }
            _ => Err(format!("unexpected or repeated flag {flag}").into()),
        }
    }

    fn path(&mut self, flag: &str, value: &str) -> Result<(), Failure> {
        let slot = if flag == "--root" { &mut self.root } else { &mut self.source };
        if slot.is_some() {
            return Err(format!("repeated flag {flag}").into());
        }
        *slot = Some(value.into());
        Ok(())
    }

    fn service(&mut self, value: &str) -> Result<(), Failure> {
        if self.services.len() >= 4096 || !self.services.insert(value.to_owned()) {
            return Err("duplicate service or service limit".into());
        }
        Ok(())
    }
}

fn value<'a>(arguments: &mut std::slice::Iter<'a, String>, flag: &str) -> Result<&'a str, Failure> {
    let value = super::next_value(arguments, flag)?;
    if value.starts_with("--") {
        return Err(format!("{flag} requires a value").into());
    }
    Ok(value)
}
