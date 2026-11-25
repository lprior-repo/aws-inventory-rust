#[cfg(test)]
mod pipeline_test_catalog;
#[cfg(test)]
mod pipeline_tests;
use generate::{ensure_fresh, generate};
use measure::measure;
use std::error::Error;
use std::path::PathBuf;

mod compare;
mod generate;
mod layout;
mod measure;
mod open;
mod sweep;

mod pipeline;
mod pipeline_fixture;
mod pipeline_native;
mod pipeline_oracle;
#[cfg(test)]
mod tests;

const USAGE: &str = "usage: cmdb-bench generate <synthetic-reader-db> <accounts> <resources-per-account> | measure <database> | pipeline <fresh-db> <compiled-catalog> <pages-per-scope> <rows-per-page:0..1000> <concurrency:1..64> [<loopback-redis-url> <fresh-cmdb_bench_graph>] | sweep --collector <path> --profile thin|full|proxy [--runs <1..100>] [--proxy latency_ms=<n>,rps=<n>] [--out <file>] [--catalog <file>] [--work <empty-dir>] [--service <recipe>]... [--server container|native|managed] [--volume named|bind|native-disk] [--build release|debug] | layout <target-nodes:1..4000000> | open --catalog <file> | compare <before> <after> [--band <percent>] [--allow-different-server]; compare exits 0: no regression, 1: regressed, 2: refused";
const MAX_RESOURCES: usize = 10_000_000;

#[derive(Debug)]
enum Command {
    Pipeline(pipeline::Workload),
    Sweep(sweep::args::SweepArgs),
    Open(open::OpenArgs),
    Compare(compare::CompareArgs),
    Generate {
        path: PathBuf,
        accounts: usize,
        per_account: usize,
    },
    Measure {
        path: PathBuf,
    },
    Layout(layout::LayoutArgs),
}

fn required(args: &mut impl Iterator<Item = String>) -> Result<String, Box<dyn Error>> {
    args.next().ok_or_else(|| USAGE.into())
}

fn parse_count(text: &str) -> Result<usize, Box<dyn Error>> {
    let value = text.parse::<usize>()?;
    if value == 0 {
        return Err(USAGE.into());
    }
    Ok(value)
}

fn parse_command(mut args: impl Iterator<Item = String>) -> Result<Command, Box<dyn Error>> {
    match required(&mut args)?.as_str() {
        "generate" => parse_generate(&mut args),
        "measure" => parse_measure(&mut args),
        "pipeline" => parse_pipeline(&mut args),
        "sweep" => Ok(Command::Sweep(sweep::args::parse(&mut args)?)),
        "layout" => Ok(Command::Layout(layout::parse(&mut args)?)),
        "open" => Ok(Command::Open(open::parse(&mut args)?)),
        "compare" => Ok(Command::Compare(compare::parse_args(&mut args)?)),
        _ => Err(USAGE.into()),
    }
}

fn parse_generate(args: &mut impl Iterator<Item = String>) -> Result<Command, Box<dyn Error>> {
    let path = PathBuf::from(required(args)?);
    let accounts = parse_count(&required(args)?)?;
    let per_account = parse_count(&required(args)?)?;
    if accounts
        .checked_mul(per_account)
        .filter(|total| *total <= MAX_RESOURCES)
        .is_none()
    {
        return Err(USAGE.into());
    }
    if args.next().is_some() {
        return Err(USAGE.into());
    }
    Ok(Command::Generate {
        path,
        accounts,
        per_account,
    })
}

fn parse_measure(args: &mut impl Iterator<Item = String>) -> Result<Command, Box<dyn Error>> {
    let path = PathBuf::from(required(args)?);
    if args.next().is_some() {
        return Err(USAGE.into());
    }
    Ok(Command::Measure { path })
}

fn parse_pipeline(args: &mut impl Iterator<Item = String>) -> Result<Command, Box<dyn Error>> {
    let path = PathBuf::from(required(args)?);
    let catalog = PathBuf::from(required(args)?);
    let pages = parse_count(&required(args)?)?;
    let rows = required(args)?.parse::<usize>()?;
    let concurrency = parse_count(&required(args)?)?;
    let graph = args
        .next()
        .map(|endpoint| -> Result<_, Box<dyn Error>> {
            Ok(pipeline_native::Target {
                endpoint,
                graph: required(args)?,
            })
        })
        .transpose()?;
    if args.next().is_some() {
        return Err(USAGE.into());
    }
    let workload = pipeline::Workload {
        path,
        catalog,
        pages,
        rows,
        concurrency,
        graph,
    };
    workload.validate().map_err(|error| error.to_string())?;
    Ok(Command::Pipeline(workload))
}

#[tokio::main]
async fn main() -> Result<std::process::ExitCode, Box<dyn Error>> {
    let mut args = std::env::args().skip(1).peekable();
    let comparing = args.peek().is_some_and(|word| word == "compare");
    let command = match parse_command(args) {
        Ok(command) => command,
        Err(error) if comparing => {
            eprintln!("comparison refused: {error}");
            return Ok(std::process::ExitCode::from(2));
        }
        Err(error) => return Err(error),
    };
    let result = match command {
        Command::Sweep(args) => benchmark_exit(sweep::run(args).await),
        Command::Open(args) => open::run(&args).await.map_err(Into::into),
        Command::Compare(args) => {
            return Ok(match compare::run(&args) {
                Ok(true) => std::process::ExitCode::SUCCESS,
                Ok(false) => std::process::ExitCode::FAILURE,
                Err(error) => {
                    eprintln!("comparison refused: {error}");
                    std::process::ExitCode::from(2)
                }
            });
        }
        Command::Pipeline(workload) => pipeline::run(workload)
            .await
            .map_err(|error| -> Box<dyn Error> { error }),
        Command::Generate {
            path,
            accounts,
            per_account,
        } => generate(path, accounts, per_account).await,
        Command::Measure { path } => measure(path).await,
        Command::Layout(args) => layout::run(&args).map_err(Into::into),
    };
    result.map(|()| std::process::ExitCode::SUCCESS)
}

fn benchmark_exit(outcome: Result<bool, String>) -> Result<(), Box<dyn Error>> {
    match outcome? {
        true => Ok(()),
        false => {
            Err("benchmark failed: collector exited nonzero or a measurement regressed".into())
        }
    }
}
