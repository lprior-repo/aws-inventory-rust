use crate::{HarnessError, postgres};
use std::{path::PathBuf, process::ExitCode};

const USAGE: &str = "usage: inventory-harness e2e run --work <empty-dir> [--postgres-url <admin-url>] [--seeds <dir>] [--service <service[:Operation]>]... [--collector <binary>] [--keep]";

pub struct RunArgs {
    pub work: PathBuf,
    pub seeds: PathBuf,
    pub catalog: PathBuf,
    pub services: Vec<String>,
    pub collector: PathBuf,
    pub keep: bool,
    admin: Option<String>,
}

impl RunArgs {
    pub fn parse(arguments: &[String]) -> Result<Self, HarnessError> {
        let Some((command, rest)) = arguments.split_first() else {
            return Err(HarnessError::invalid(USAGE));
        };
        if command != "run" {
            return Err(HarnessError::invalid(USAGE));
        }
        Self::parse_options(rest)
    }

    fn defaults() -> Result<Self, HarnessError> {
        let root = super::workspace_root();
        let executable = std::env::current_exe()?;
        let parent = executable
            .parent()
            .ok_or_else(|| HarnessError::invalid("cannot locate collector directory"))?;
        Ok(Self {
            work: PathBuf::new(),
            seeds: root.join("seeds"),
            catalog: root.join("catalog/aws-api.catalog"),
            services: Vec::new(),
            collector: parent.join("aws-inventory"),
            keep: false,
            admin: None,
        })
    }

    fn parse_options(rest: &[String]) -> Result<Self, HarnessError> {
        let initial = Self::defaults()?;
        let mut values = rest.iter();
        let args = (0..rest.len()).try_fold(initial, |mut args, _| {
            if let Some(option) = values.next() {
                args.option(option, &mut values)?;
            }
            Ok::<_, HarnessError>(args)
        })?;
        if args.work.as_os_str().is_empty() {
            return Err(HarnessError::invalid(USAGE));
        }
        Ok(args)
    }

    fn option<'a>(
        &mut self,
        name: &str,
        values: &mut impl Iterator<Item = &'a String>,
    ) -> Result<(), HarnessError> {
        if name == "--keep" {
            self.keep = true;
            return Ok(());
        }
        let value = values
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| HarnessError::invalid(USAGE))?;
        match name {
            "--work" => self.work = PathBuf::from(value),
            "--seeds" => self.seeds = PathBuf::from(value),
            "--collector" => self.collector = PathBuf::from(value),
            "--service" => self.services.push(value.clone()),
            "--postgres-url" => self.admin = Some(value.clone()),
            _ => return Err(HarnessError::invalid(USAGE)),
        }
        Ok(())
    }

    fn admin_url(&self) -> Result<String, HarnessError> {
        match &self.admin {
            Some(url) => Ok(url.clone()),
            None => postgres::admin_url(),
        }
    }
}

pub async fn run(arguments: &[String]) -> ExitCode {
    let configured =
        RunArgs::parse(arguments).and_then(|args| args.admin_url().map(|admin| (args, admin)));
    let (args, admin) = match configured {
        Ok(configured) => configured,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    match super::execute(args, &admin).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("e2e run failed: {error}");
            ExitCode::FAILURE
        }
    }
}
