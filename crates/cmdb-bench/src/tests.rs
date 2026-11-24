use super::{Command, ensure_fresh, generate, measure, parse_command};
use cmdb_view::{Database, PageRequest, ResourceFilter};
use std::error::Error;
use tempfile::tempdir;

type TestResult = Result<(), Box<dyn Error>>;

#[tokio::test]
async fn generated_binary_database_round_trips_through_reader() -> TestResult {
    let directory = tempdir()?;
    let path = directory.path().join("benchmark.sqlite");
    generate(path.clone(), 2, 3).await?;
    let database = Database::open(&path).await?;
    let counts = database.counts().await?;
    assert_eq!(counts.accounts, 2);
    assert_eq!(counts.regions, 3);
    assert_eq!(counts.services, 2);
    assert_eq!(counts.resources, 6);
    let page = database
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 6,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.resources.len(), 6);
    assert_eq!(page.total_matching, 6);
    assert!(page.next.is_none());
    assert_eq!(
        page.resources
            .first()
            .map(|resource| resource.key.kind.as_str()),
        Some("AWS::EC2::Instance")
    );
    measure(path).await?;
    Ok(())
}

#[test]
fn command_parser_rejects_zero_and_extra_arguments() {
    assert!(
        parse_command(
            ["generate", "out.sqlite", "0", "3"]
                .map(str::to_owned)
                .into_iter()
        )
        .is_err()
    );
    assert!(
        parse_command(
            ["measure", "out.sqlite", "extra"]
                .map(str::to_owned)
                .into_iter()
        )
        .is_err()
    );
}

#[test]
fn parser_preserves_explicit_workload_parameters() -> TestResult {
    let command = parse_command(
        ["generate", "work.sqlite", "4", "25"]
            .map(str::to_owned)
            .into_iter(),
    )?;
    assert!(matches!(
        command,
        Command::Generate {
            accounts: 4,
            per_account: 25,
            ..
        }
    ));
    Ok(())
}

#[test]
fn generator_refuses_nonempty_existing_database() -> TestResult {
    let directory = tempdir()?;
    let path = directory.path().join("existing.sqlite");
    std::fs::write(&path, b"preserve existing")?;
    assert!(ensure_fresh(&path).is_err());
    Ok(())
}

#[test]
fn layout_command_parses_target_nodes() -> TestResult {
    let command = parse_command(["layout", "1000"].map(str::to_owned).into_iter())?;
    assert!(matches!(command, Command::Layout(_)));
    Ok(())
}

#[test]
fn layout_command_rejects_zero_and_extra_arguments() {
    assert!(parse_command(["layout", "0"].map(str::to_owned).into_iter()).is_err());
    assert!(parse_command(["layout", "10", "extra"].map(str::to_owned).into_iter()).is_err());
}
