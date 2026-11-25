use super::{
    context::Context,
    server::{Server, TestResult},
};
use inventory_core::catalog::runtime::RuntimeCatalog;
use std::time::Duration;

pub async fn twice(catalog: &RuntimeCatalog<'_>) -> TestResult<(String, Vec<String>)> {
    let server = Server::new().await?;
    let context = Context::new(&server)?;
    let probe = async {
        let first = context.report(catalog).await?;
        let second = context.report(catalog).await?;
        assert_eq!(first, second);
        Ok::<_, Box<dyn std::error::Error>>(first)
    };
    tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(probe, server.serve(10))
    })
    .await?
}

pub async fn once(
    catalog: &RuntimeCatalog<'_>,
) -> TestResult<(inventory_sweep::ProbeAnswer, Vec<String>)> {
    let server = Server::new().await?;
    let context = Context::new(&server)?;
    let probe = async {
        Ok::<_, Box<dyn std::error::Error>>(context.once(catalog, "ListParents0", None).await?)
    };
    tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(probe, server.serve(1))
    })
    .await?
}
