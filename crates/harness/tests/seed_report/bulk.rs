use super::*;
use inventory_harness::inventory_seed::{command::SeedRunArgs, run_report_with_concurrency};

const TOPICS: &str = "format=1\nservice='sns'\n[[step]]\nid='topic'\noperation='CreateTopic'\nkind='AWS::SNS::Topic'\nidentity='TopicArn'\nrepeat=8\n[step.input]\nName='inventory-concurrent-{n}'\n";

async fn execute(concurrency: usize) -> Result<(SeedRunReport, usize)> {
    let server = Server::new().await?;
    let config = SeedConfig::new(
        &format!("https://{}", server.listener.local_addr()?),
        &server.ca,
        "us-east-1",
    )?;
    let bytes = std::fs::read(catalog_path())?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let credentials = Credentials::new("SEEDREPORT", "test-secret", None)?;
    let scenario = load(TOPICS)?;
    let (report, peak) = tokio::time::timeout(Duration::from_secs(15), async {
        tokio::join!(
            run_report_with_concurrency(&catalog, &config, &credentials, &scenario, concurrency),
            server.serve_topics(8)
        )
    })
    .await?;
    Ok((report?, peak?))
}

#[tokio::test]
async fn given_repeated_steps_when_concurrency_is_three_then_flight_is_bounded_and_oracle_is_deterministic()
-> Result {
    let sequential = execute(1).await?;
    let concurrent = execute(3).await?;
    assert_eq!(sequential.1, 1);
    assert_eq!(concurrent.1, 3);
    assert_eq!(sequential.0.oracle.resources.len(), 8);
    assert_eq!(sequential.0.oracle_rows(), concurrent.0.oracle_rows());
    assert!(
        concurrent
            .0
            .outcomes
            .iter()
            .all(|outcome| matches!(outcome, StepOutcome::Accepted))
    );
    Ok(())
}

#[test]
fn given_invalid_concurrency_when_parsing_then_zero_oversize_and_nonintegers_fail() -> Result {
    let arguments = arguments();
    assert_eq!(SeedRunArgs::parse(&arguments)?.concurrency, 1);
    for value in ["0", "65", "-1", "invalid"] {
        let mut supplied = arguments.clone();
        supplied.extend(["--concurrency".into(), value.into()]);
        assert!(SeedRunArgs::parse(&supplied).is_err());
    }
    let mut supplied = arguments;
    supplied.extend(["--concurrency".into(), "64".into()]);
    assert_eq!(SeedRunArgs::parse(&supplied)?.concurrency, 64);
    Ok(())
}

fn arguments() -> Vec<String> {
    [
        "--seeds",
        "seeds",
        "--catalog",
        "catalog",
        "--endpoint",
        "https://127.0.0.1:443",
        "--ca-bundle",
        "ca",
        "--region",
        "us-east-1",
        "--oracle",
        "oracle",
        "--matrix",
        "matrix",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
