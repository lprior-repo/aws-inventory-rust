use super::*;
#[tokio::test]
async fn given_legacy_store_without_catalog_when_plain_open_commits_unrelated_observation_then_no_implicit_relationship_authority_is_granted()
-> TestResult {
    use inventory_core::inventory::Edge;
    let bytes = oam_fixture::compiled(
        "oam",
        "2022-06-10",
        &["ListLinks", "ListSinks"],
        "AWS::Oam::Link",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for version in [14_i64, 15] {
        let (directory, original) = store(&catalog).await?;
        persist_oam(&original, &catalog, "ListLinks", "123456789012", 1).await?;
        drop(original);
        let path = directory.path().join("inventory.sqlite");
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        let mut tx = pool.begin().await?;
        downgrade_oam(&mut tx, version).await?;
        tx.commit().await?;
        pool.close().await;
        // Given no reviewed catalog at all, empty proof storage is bookkeeping, not permission.
        let reopened = BinaryStore::open(&path).await?;
        persist_oam(&reopened, &catalog, "ListSinks", "210987654321", 2).await?;
        assert_eq!(
            reopened.projection_resources(None, 100).await?.len(),
            2,
            "schema {version}"
        );
        assert!(reopened.projection_edges(None, 100).await?.edges.is_empty());
        for direct in [false, true] {
            let mut page = oam_page(
                &reopened,
                &catalog,
                "ListLinks",
                "123456789012",
                if direct { 4 } else { 3 },
            )
            .await?;
            if direct {
                let reference = page.unresolved.first().ok_or("OAM reference absent")?;
                page.edges.push(Edge {
                    from: reference.from.clone(),
                    to: ResourceKey {
                        account: reference.target.account().clone(),
                        region: reference.target.region().clone(),
                        kind: reference.target.kind().clone(),
                        id: reference.target.value().clone(),
                    },
                    relation: reference.relation.clone(),
                    source: reference.source,
                });
                page.unresolved.clear();
            }
            let lease = page.lease.clone();
            assert!(
                matches!(
                    reopened.commit_page(page).await,
                    Err(inventory_engine::inventory::StoreError::Invalid(
                        "relationship evidence is not permitted by the checked catalog"
                    ))
                ),
                "schema {version}, direct {direct}"
            );
            reopened
                .finish_scope(lease, inventory_core::inventory::ScopeOutcome::Failed, 10)
                .await?;
            assert_eq!(reopened.projection_resources(None, 100).await?.len(), 2);
            assert!(reopened.projection_edges(None, 100).await?.edges.is_empty());
        }
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM metadata WHERE key='reviewed_catalog'"
            )
            .fetch_one(&pool)
            .await?,
            0
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_grants")
                .fetch_one(&pool)
                .await?,
            0
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_evidence")
                .fetch_one(&pool)
                .await?,
            0
        );
        pool.close().await;
    }
    Ok(())
}
