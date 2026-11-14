use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn list_accounts(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
) -> Listing<OrganizationAccount> {
    pages::collect(
        client,
        "organizations:ListAccounts",
        None,
        MAX_DISCOVERED_ACCOUNTS,
        |token| async move {
            let response = client
                .execute("organizations", true, || async {
                    sdk.list_accounts()
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 20))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.accounts.unwrap_or_default(),
                response.next_token,
                map_account,
            )
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn list_roots(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
) -> Listing<OrganizationNode> {
    pages::collect(
        client,
        "organizations:ListRoots",
        None,
        MAX_ROOTS,
        |token| async move {
            let response = client
                .execute("organizations", true, || async {
                    sdk.list_roots()
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 20))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.roots.unwrap_or_default(),
                response.next_token,
                map_root,
            )
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(parent = %parent.id))]
pub(super) async fn list_units(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
    parent: &OrganizationNode,
    remaining: usize,
) -> Listing<OrganizationNode> {
    pages::collect(
        client,
        "organizations:ListOrganizationalUnitsForParent",
        Some(&parent.id),
        remaining,
        |token| async move {
            let response = client
                .execute("organizations", true, || async {
                    sdk.list_organizational_units_for_parent()
                        .parent_id(&parent.id)
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 20))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.organizational_units.unwrap_or_default(),
                response.next_token,
                |unit| map_unit(unit, &parent.id, parent.depth.saturating_add(1)),
            )
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(parent))]
pub(super) async fn list_members(
    client: &DiscoveryClient<'_>,
    sdk: &aws_sdk_organizations::Client,
    parent: &str,
    remaining: usize,
) -> Listing<String> {
    pages::collect(
        client,
        "organizations:ListChildren",
        Some(parent),
        remaining,
        |token| async move {
            let response = client
                .execute("organizations", true, || async {
                    sdk.list_children()
                        .parent_id(parent)
                        .child_type(ChildType::Account)
                        .set_next_token(token.clone())
                        .max_results(client.page_size.clamp(1, 20))
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            Page::map(
                response.children.unwrap_or_default(),
                response.next_token,
                map_member,
            )
        },
    )
    .await
}
