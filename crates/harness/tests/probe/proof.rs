use super::{
    context::Context,
    server::{Server, TestResult},
};
use inventory_core::catalog::runtime::RuntimeCatalog;

pub async fn twice(catalog: &RuntimeCatalog<'_>) -> TestResult<(String, Vec<String>)> {
    let server = Server::new().await?;
    let context = Context::new(&server)?;
    let probe = async {
        let first = context.report(catalog).await?;
        let second = context.report(catalog).await?;
        assert_eq!(first, second);
        Ok::<_, Box<dyn std::error::Error>>(first)
    };
    server.capture_with(probe, super::server::response).await
}

pub async fn once(
    catalog: &RuntimeCatalog<'_>,
) -> TestResult<(inventory_sweep::ProbeAnswer, Vec<String>)> {
    let server = Server::new().await?;
    let context = Context::new(&server)?;
    let probe = async {
        Ok::<_, Box<dyn std::error::Error>>(context.once(catalog, "ListParents0", None).await?)
    };
    server.capture_with(probe, super::server::response).await
}
