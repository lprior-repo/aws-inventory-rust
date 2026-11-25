use super::{
    context::Context,
    fixture,
    server::{Server, TestResult},
};
use catalog_tool::overlay::{Binding, MemberPath};
use inventory_core::{catalog::runtime::RuntimeCatalog, inventory::AccountId};

fn catalog_bytes() -> TestResult<Vec<u8>> {
    let account_path = MemberPath::new("OrganizationId")?;
    fixture::compiled_with_parents(
        1,
        |parent| {
            parent.fixed.remove(&account_path);
            parent.bind.insert(account_path.clone(), Binding::Account);
        },
        |_| {},
    )
}

async fn probe(
    catalog: &RuntimeCatalog<'_>,
    server: &Server,
    account: &AccountId,
) -> TestResult<(inventory_sweep::ProbeAnswer, Vec<String>)> {
    let context = Context::new(server)?;
    let call = async {
        context
            .once(catalog, "ListParents0", Some(account))
            .await
            .map_err(Into::into)
    };
    server.capture_with(call, super::server::response).await
}

#[tokio::test]
async fn given_account_bound_root_when_probed_then_signed_body_contains_real_account() -> TestResult
{
    let bytes = catalog_bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let server = Server::new().await?;
    let account = AccountId::parse("111122223333")?;
    let (answer, requests) = probe(&catalog, &server, &account).await?;
    assert_eq!(answer.items, 2);
    assert_eq!(requests.len(), 1);
    assert!(
        requests
            .first()
            .is_some_and(|request| request.contains(r#""OrganizationId":"111122223333""#))
    );
    Ok(())
}

#[tokio::test]
async fn given_account_bound_root_without_account_when_probed_then_no_request_is_sent() -> TestResult
{
    let bytes = catalog_bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let server = Server::new().await?;
    let context = Context::new(&server)?;
    let result = context.once(&catalog, "ListParents0", None).await;
    assert!(matches!(
        result,
        Err(inventory_sweep::SweepError::Operation(
            "account binding unavailable during bootstrap"
        ))
    ));
    Ok(())
}
