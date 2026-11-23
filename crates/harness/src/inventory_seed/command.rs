mod args;
mod directory;

use super::SeedError;
pub use args::SeedRunArgs;
use aws_wire::sign::Credentials;
use inventory_core::catalog::runtime::RuntimeCatalog;
use std::process::ExitCode;

pub async fn run(arguments: &[String]) -> ExitCode {
    match execute(arguments).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn execute(arguments: &[String]) -> Result<(), SeedError> {
    let args = SeedRunArgs::parse(arguments)?;
    let config = super::SeedConfig::new(&args.endpoint, &args.ca_bundle, &args.region)?;
    let credentials = credentials()?;
    let bytes = directory::read_bounded(&args.catalog, 67_108_864)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (oracle, matrix) = directory::run(&args, &catalog, &config, &credentials).await?;
    directory::write_rows(&args.oracle, oracle)?;
    directory::write_rows(&args.matrix, matrix)?;
    Ok(())
}

fn credentials() -> Result<Credentials, SeedError> {
    let key = std::env::var("AWS_ACCESS_KEY_ID")
        .map_err(|_| SeedError::Invalid("seed run requires AWS_ACCESS_KEY_ID"))?;
    let secret = std::env::var("AWS_SECRET_ACCESS_KEY")
        .map_err(|_| SeedError::Invalid("seed run requires AWS_SECRET_ACCESS_KEY"))?;
    let token = match std::env::var("AWS_SESSION_TOKEN") {
        Ok(token) => Some(token),
        Err(std::env::VarError::NotPresent) => None,
        Err(_) => return Err(SeedError::Invalid("AWS_SESSION_TOKEN must be UTF-8")),
    };
    Ok(Credentials::new(&key, &secret, token.as_deref())?)
}
