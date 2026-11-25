use super::*;

pub(super) async fn denied_preserves_inventory(
    environment: &Environment,
    database: &Database,
    oracle: &TypedSeedOracle,
    before: &InventorySnapshot,
    peer: &AccessDeniedPeer,
) -> Result<()> {
    let peer_url = peer.endpoint();
    let endpoint = EndpointOverride {
        url: &peer_url,
        ca_bundle: peer.ca_bundle(),
    };
    let arguments = sweep_args_for(environment, database, 100, false, Some(endpoint))?;
    for _ in 0..2 {
        let output = run_cli_degraded(environment, &arguments).await?;
        verify_denied(&output)?;
        ensure!(
            &database_snapshot(database, oracle).await? == before,
            "repeated denied runs changed resources, edges, or missing counters"
        );
    }
    Ok(())
}

fn verify_denied(summary: &str) -> Result<()> {
    ensure!(
        summary_value(summary, "inaccessible")? > 0,
        "controlled AWS 403 responses must become inaccessible scopes"
    );
    assert_no_failures(summary)?;
    ensure!(
        summary_value(summary, "resource_writes")? == 0,
        "denied scopes must not write inventory resources"
    );
    Ok(())
}

pub(super) struct EndpointOverride<'a> {
    pub(super) url: &'a str,
    pub(super) ca_bundle: &'a Path,
}

pub(super) fn sweep_args(
    environment: &Environment,
    database: &Database,
    max_pages: u32,
    resume: bool,
) -> Result<Vec<String>> {
    sweep_args_for(environment, database, max_pages, resume, None)
}

pub(super) fn sweep_args_for(
    environment: &Environment,
    database: &Database,
    max_pages: u32,
    resume: bool,
    endpoint: Option<EndpointOverride<'_>>,
) -> Result<Vec<String>> {
    let mut arguments = Vec::with_capacity(22);
    identity_args(&mut arguments, environment, database)?;
    network_args(&mut arguments, environment, endpoint)?;
    arguments.extend([
        "--max-pages".to_owned(),
        max_pages.to_string(),
        "--max-page-size".to_owned(),
        "5".to_string(),
        "--concurrency".to_owned(),
        "4".to_owned(),
    ]);
    if resume {
        arguments.push("--resume".to_owned());
    }
    Ok(arguments)
}

fn identity_args(
    arguments: &mut Vec<String>,
    environment: &Environment,
    database: &Database,
) -> Result<()> {
    arguments.extend([
        "sweep".to_owned(),
        "--database".to_owned(),
        database.argument()?,
        "--catalog".to_owned(),
        text_path(&environment.catalog)?,
        "--profile".to_owned(),
        environment.profile.clone(),
        "--region".to_owned(),
        environment.region.clone(),
        "--service".to_owned(),
        "ec2:DescribeVpcs".to_owned(),
        "--service".to_owned(),
        "ec2:DescribeSubnets".to_owned(),
    ]);
    Ok(())
}

fn network_args(
    arguments: &mut Vec<String>,
    environment: &Environment,
    endpoint: Option<EndpointOverride<'_>>,
) -> Result<()> {
    let endpoint_url = endpoint
        .as_ref()
        .map_or(environment.endpoint.as_str(), |value| value.url);
    let ca_bundle = endpoint
        .as_ref()
        .map_or(environment.ca_bundle.as_path(), |value| value.ca_bundle);
    arguments.extend([
        "--endpoint".to_owned(),
        endpoint_url.to_owned(),
        "--ca-bundle".to_owned(),
        text_path(ca_bundle)?,
        "--graph-url".to_owned(),
        environment.graph_url.clone(),
        "--graph-name".to_owned(),
        environment.graph_name.clone(),
    ]);
    Ok(())
}

pub(super) fn set_concurrency(arguments: &mut [String], concurrency: usize) -> Result<()> {
    let position = arguments
        .iter()
        .position(|value| value == "--concurrency")
        .context("E2E command omitted concurrency option")?;
    *arguments
        .get_mut(
            position
                .checked_add(1)
                .context("E2E concurrency option position overflow")?,
        )
        .context("E2E concurrency value is missing")? = concurrency.to_string();
    Ok(())
}

pub(super) fn text_path(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .context("E2E paths must be UTF-8")
}

pub(super) fn summary_value(summary: &str, name: &str) -> Result<u64> {
    summary
        .split_whitespace()
        .find_map(|item| item.strip_prefix(&format!("{name}="))?.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("sweep summary omitted {name}=<count>"))
}

pub(super) fn assert_no_failures(summary: &str) -> Result<()> {
    ensure!(
        summary_value(summary, "failed")? == 0,
        "sweep CLI summary contains fatal failed scopes"
    );
    Ok(())
}

pub(super) fn assert_baseline_budget(summary: &str) -> Result<()> {
    for (name, expected) in [
        ("scopes", 2),
        ("complete", 2),
        ("pages", 4),
        ("api_attempts", 6),
        ("api_retries", 0),
        ("unsupported", 0),
    ] {
        ensure!(
            summary_value(summary, name)? == expected,
            "selected VPC/subnet baseline exceeded {name} budget {expected}: {summary}"
        );
    }
    Ok(())
}

pub(super) fn closed_loopback_url() -> Result<String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let address = listener.local_addr()?;
    drop(listener);
    Ok(format!("redis://{address}"))
}
