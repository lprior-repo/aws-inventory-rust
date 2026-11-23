use std::process::ExitCode;

use super::{admin_url, database, prune, runtime};
use crate::HarnessError;

const USAGE: &str = "inventory-harness postgres up [--runtime docker|podman] | down [--runtime docker|podman] [--volumes] | url | prune";
const DEFAULT_URL: &str = "postgres://inventory:inventory-local-only@127.0.0.1:54329/inventory";

pub async fn run(arguments: &[String]) -> ExitCode {
    match execute(arguments).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

async fn execute(arguments: &[String]) -> Result<(), HarnessError> {
    match arguments {
        [action, rest @ ..] if action == "up" => {
            let (runtime, action) = parse_runtime(rest, runtime::Action::Up)?;
            runtime::compose(runtime, action).await?;
            println!("{}", local_url()?);
        }
        [action, rest @ ..] if action == "down" => {
            let (runtime, action) = parse_runtime(rest, runtime::Action::Down)?;
            runtime::compose(runtime, action).await?;
        }
        [action] if action == "url" => println!("{}", local_url()?),
        [action] if action == "prune" => {
            println!("pruned {} databases", prune(&admin_url()?, 3_600).await?)
        }
        _ => return Err(HarnessError::invalid(USAGE)),
    }
    Ok(())
}

fn parse_runtime(
    arguments: &[String],
    action: runtime::Action,
) -> Result<(Option<runtime::Runtime>, runtime::Action), HarnessError> {
    let (arguments, action) = volume_option(arguments, action)?;
    match arguments {
        [] => Ok((None, action)),
        [flag, name] if flag == "--runtime" => Ok((Some(runtime::Runtime::parse(name)?), action)),
        _ => Err(HarnessError::invalid(USAGE)),
    }
}

fn volume_option(
    arguments: &[String],
    action: runtime::Action,
) -> Result<(&[String], runtime::Action), HarnessError> {
    let remaining = match arguments {
        [flag, rest @ ..] if flag == "--volumes" => rest,
        [rest @ .., flag] if flag == "--volumes" => rest,
        _ => return Ok((arguments, action)),
    };
    if !matches!(action, runtime::Action::Down) {
        return Err(HarnessError::invalid(USAGE));
    }
    Ok((remaining, runtime::Action::DownWithVolumes))
}

fn local_url() -> Result<String, HarnessError> {
    if std::env::var_os("INVENTORY_POSTGRES_URL").is_some() {
        return admin_url();
    }
    match std::env::var("INVENTORY_POSTGRES_PASSWORD") {
        Ok(password) => {
            if password.len() > 1_024 {
                return Err(HarnessError::invalid(
                    "local PostgreSQL password exceeds 1024 bytes",
                ));
            }
            use sqlx::ConnectOptions;
            Ok(database::options(DEFAULT_URL)?
                .password(&password)
                .to_url_lossy()
                .into())
        }
        Err(std::env::VarError::NotPresent) => Ok(DEFAULT_URL.to_owned()),
        Err(_) => Err(HarnessError::invalid(
            "local PostgreSQL password must be Unicode",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_commands_reject_unknown_options() {
        let invalid = ["--runtime".to_owned(), "invalid".to_owned()];
        assert!(parse_runtime(&invalid, runtime::Action::Up).is_err());
        assert!(parse_runtime(&["--volumes".to_owned()], runtime::Action::Up).is_err());
        assert!(parse_runtime(&["--volumes".to_owned()], runtime::Action::Down).is_ok());
    }
}
