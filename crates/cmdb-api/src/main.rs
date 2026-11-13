use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match cmdb_api::run(std::env::args().skip(1)).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.is_help() => {
            print!("{}", cmdb_api::USAGE);
            ExitCode::SUCCESS
        }
        Err(error) => {
            cmdb_api::report(&error);
            ExitCode::FAILURE
        }
    }
}
