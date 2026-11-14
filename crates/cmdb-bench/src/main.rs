//! Deterministic fleet workload generator and oracle checker.
#![deny(missing_docs)]

mod deletions;
mod dist;
mod edges;
mod generator;
mod layout;
mod model;
mod options;
mod pages;
mod plan;
mod records;
mod rng;
mod sink;
mod skeleton;
#[cfg(test)]
mod tests;
mod verify;

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use options::{Options, SizeSpec};

const USAGE: &str = "usage:\n  cmdb-bench workload --accounts <n> (--per-account <n> | --assets-total <n>) [options]\n\
    --seed <n>             RNG seed (default 1)\n\
    --output <dir>         output directory (default workload)\n\
    --empty-regions <n>    unused regions recorded by the oracle (default 0)\n\
    --global-share <pct>   percent of accounts with an injected global service (default 0)\n\
    --payload-bytes <n>    maximum payload length (default 256)\n\
    --duplicate-pct <pct>  percent of events emitted twice (default 0)\n\
    --reorder-pct <pct>    percent of events swapped with the following event (default 0)\n\
    --deletion-bursts <n>  number of deletion bursts (default 0)\n\
    --deletion-burst-size <n> records per deletion burst (default 0)\n\n  cmdb-bench verify --input <dir>";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut arguments = arguments.into_iter();
    match arguments.next().as_deref() {
        Some("workload") => run_workload(&mut arguments),
        Some("verify") => run_verify(&mut arguments),
        _ => Err(USAGE.into()),
    }
}

#[derive(Default)]
struct WorkloadArgs {
    accounts: Option<usize>,
    per_account: Option<usize>,
    assets_total: Option<usize>,
    output: Option<PathBuf>,
}

impl WorkloadArgs {
    fn size(&self) -> Result<SizeSpec, Box<dyn Error>> {
        match (self.per_account, self.assets_total) {
            (Some(per_account), None) => Ok(SizeSpec::Uniform { per_account }),
            (None, Some(assets_total)) => Ok(SizeSpec::AssetsTotal { assets_total }),
            (Some(_), Some(_)) => {
                Err("--per-account and --assets-total are mutually exclusive".into())
            }
            (None, None) => {
                Err(format!("one of --per-account or --assets-total is required\n{USAGE}").into())
            }
        }
    }

    fn output(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from("workload"))
    }
}

fn run_workload(arguments: &mut impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut options = Options::default();
    let mut args = WorkloadArgs::default();
    parse_workload(arguments, &mut options, &mut args)?;
    options.accounts = args.accounts.ok_or(USAGE)?;
    options.size = args.size()?;
    options.validate()?;
    let oracle = generator::generate(&args.output(), options)?;
    println!("{}", serde_json::to_string(&oracle)?);
    Ok(())
}

fn parse_workload(
    arguments: &mut impl Iterator<Item = String>,
    opts: &mut Options,
    args: &mut WorkloadArgs,
) -> Result<(), Box<dyn Error>> {
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--accounts" => args.accounts = Some(parse_number(arguments.next())?),
            "--per-account" => args.per_account = Some(parse_number(arguments.next())?),
            "--assets-total" => args.assets_total = Some(parse_number(arguments.next())?),
            "--seed" => opts.seed = parse_u64(arguments.next())?,
            "--output" => args.output = Some(PathBuf::from(require(arguments.next())?)),
            "--empty-regions" => opts.empty_regions = parse_number(arguments.next())?,
            "--global-share" => opts.global_share = parse_u64(arguments.next())?,
            "--payload-bytes" => opts.payload_bytes = parse_number(arguments.next())?,
            "--duplicate-pct" => opts.duplicate_pct = parse_u64(arguments.next())?,
            "--reorder-pct" => opts.reorder_pct = parse_u64(arguments.next())?,
            "--deletion-bursts" => opts.deletion_bursts = parse_number(arguments.next())?,
            "--deletion-burst-size" => opts.deletion_burst_size = parse_number(arguments.next())?,
            other => return Err(format!("unexpected {other}; {USAGE}").into()),
        }
    }
    Ok(())
}

fn run_verify(arguments: &mut impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut input = None;
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--input" => input = Some(PathBuf::from(require(arguments.next())?)),
            other => return Err(format!("unexpected {other}; {USAGE}").into()),
        }
    }
    let input = input.ok_or(USAGE)?;
    println!("{}", serde_json::to_string(&verify::verify(&input)?)?);
    Ok(())
}

fn require(value: Option<String>) -> Result<String, Box<dyn Error>> {
    value.ok_or_else(|| USAGE.into())
}

fn parse_number(value: Option<String>) -> Result<usize, Box<dyn Error>> {
    require(value)?.parse::<usize>().map_err(Into::into)
}

fn parse_u64(value: Option<String>) -> Result<u64, Box<dyn Error>> {
    require(value)?.parse::<u64>().map_err(Into::into)
}
