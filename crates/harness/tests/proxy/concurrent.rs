use super::{
    clock::within,
    probability,
    support::{OK, Server, TestResult, finished, reached, request},
    wire,
};
use inventory_harness::proxy::{Proxy, Scope, Snapshot};
use tokio::{io::AsyncWriteExt, net::TcpStream};

type Responses = (Vec<u8>, Vec<u8>);

#[tokio::test]
async fn concurrent_scopes_have_identical_snapshots_in_both_arrival_orders() -> TestResult {
    within("two concurrent scope arrival orders", scenario()).await
}

async fn scenario() -> TestResult {
    let server = Server::start(OK).await?;
    let forward = run(&server, "a", "b").await?;
    let reverse = run(&server, "b", "a").await?;
    assert_eq!(forward, reverse);
    assert_eq!(forward.dropped, 0);
    Ok(())
}

async fn run(
    server: &Server,
    first: &str,
    second: &str,
) -> Result<Snapshot, Box<dyn std::error::Error>> {
    let proxy = Proxy::start(probability::config(server, 7)?).await?;
    let mut left = TcpStream::connect(proxy.address()).await?;
    let mut right = TcpStream::connect(proxy.address()).await?;
    left.write_all(&request(first, "us-east-1", "s3")).await?;
    reached(&proxy, 1).await?;
    right.write_all(&request(second, "us-east-1", "s3")).await?;
    let (left, right) = responses(left, right).await?;
    finished(&proxy, 2).await?;
    let snapshot = proxy.snapshot()?;
    observe(&snapshot, first, &left)?;
    observe(&snapshot, second, &right)?;
    proxy.shutdown().await?;
    Ok(snapshot)
}

async fn responses(
    left: TcpStream,
    right: TcpStream,
) -> Result<Responses, Box<dyn std::error::Error>> {
    let (left, right) = tokio::join!(wire::remaining(left), wire::remaining(right));
    Ok((left?, right?))
}

fn observe(snapshot: &Snapshot, key: &str, bytes: &[u8]) -> TestResult {
    let entry = snapshot.decisions.iter().find(
        |entry| matches!(&entry.scope, Scope::Signed { account, .. } if account.as_str() == key),
    );
    probability::observe(entry.ok_or("missing concurrent decision")?, bytes)
}
