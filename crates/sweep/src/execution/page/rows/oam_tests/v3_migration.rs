use super::*;
#[path = "v3_fixture.rs"]
mod fixture;
use fixture::{catalog_bytes, facet_row, frozen, old_image, page, partial, pool};
use inventory_core::inventory::ScopeOutcome;

#[path = "v3_migration/authority.rs"]
mod authority;
#[path = "v3_replay.rs"]
mod replay;

#[tokio::test]
async fn given_honest_v2_discovery_facet_when_same_catalog_targets_and_partial_sources_arrive_then_only_two_fresh_misses_and_metadata_complete_release_it()
-> TestResult {
    let bytes = catalog_bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for complete_before_misses in [false, true] {
        let (directory, original) = store(&catalog).await?;
        let old = page(&original, &catalog, "metadata", "old-metadata", 1, true).await?;
        let reference = fixture::historical_evidence(
            &catalog,
            old.unresolved
                .first()
                .ok_or("native OAM reference absent")?,
        )?;
        original.commit_page(old).await?;
        original
            .commit_page(page(&original, &catalog, "discover", "old-discover", 2, false).await?)
            .await?;
        drop(original);
        // The historical writer accepted a source under a DiscoveryOnly stage. Rebuild
        // its actual seven-field codec and v1 hash, not a modern fabricated proof.
        old_image(&directory, &reference).await?;
        let path = directory.path().join("inventory.sqlite");
        let sql = pool(&directory).await?;
        let before = frozen(&sql, &catalog).await?;
        let digest: Vec<u8> =
            sqlx::query_scalar("SELECT value FROM metadata WHERE key='catalog_digest'")
                .fetch_one(&sql)
                .await?;
        let old_facet = facet_row(&sql, false).await?;
        sql.close().await;
        let store = BinaryStore::open_with_catalog(&path, &bytes).await?;
        assert_eq!(store.unattributed_authority_count().await?, 1);
        // Reusing the OLD sweep name for a genuine unrelated target must not make
        // the old terminal DiscoveryOnly outcome a current negative observation.
        let target = oam_page_in_sweep(
            &store,
            &catalog,
            "ListSinks",
            "210987654321",
            3,
            ("us-east-1", LINK, SINK),
            SweepId::new("old-discover".to_owned())?,
        )
        .await?;
        store.commit_page(target).await?;
        let sql = pool(&directory).await?;
        assert_eq!(frozen(&sql, &catalog).await?, before);
        assert_eq!(
            sqlx::query_scalar::<_, Vec<u8>>(
                "SELECT value FROM metadata WHERE key='catalog_digest'"
            )
            .fetch_one(&sql)
            .await?,
            digest
        );
        assert_eq!(facet_row(&sql, true).await?, old_facet);
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM archived_reviewed_reference_grants_pre_v3"
            )
            .fetch_one(&sql)
            .await?,
            1
        );
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_evidence WHERE authority IS NOT NULL AND from_id=?1").bind(LINK).fetch_one(&sql).await?, 0);
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT v3_misses FROM legacy_authorities WHERE id=?1")
                .bind(LINK)
                .fetch_one(&sql)
                .await?,
            0
        );
        assert!(store.projection_edges(None, 100).await?.edges.is_empty());
        sql.close().await;
        drop(store);
        let store = BinaryStore::open_with_catalog(&path, &bytes).await?;
        // Checked DiscoveryOnly is still not an emitting stage, even for this exact
        // catalog, operation, bindings, source kind and reviewed relationship tuple.
        let forged = page(&store, &catalog, "discover", "forged-discover", 10, true).await?;
        let lease = forged.lease.clone();
        assert!(store.commit_page(forged).await.is_err());
        store.finish_scope(lease, ScopeOutcome::Partial, 11).await?;
        // A different genuine Metadata operation cannot certify a Link baseline
        // merely because the supplied Link observation is itself schema-valid.
        let link = page(
            &store,
            &catalog,
            "metadata",
            "wrong-operation-source",
            12,
            true,
        )
        .await?;
        let observations = link.observations;
        store
            .finish_scope(link.lease, ScopeOutcome::Partial, 13)
            .await?;
        let own_sink = SINK.replace("210987654321", "123456789012");
        let mut wrong_operation = oam_page_in_scope(
            &store,
            &catalog,
            "ListSinks",
            "123456789012",
            14,
            ("us-east-1", LINK, &own_sink),
        )
        .await?;
        wrong_operation.observations = observations;
        wrong_operation.unresolved.clear();
        let lease = wrong_operation.lease.clone();
        assert!(store.commit_page(wrong_operation).await.is_err());
        store.finish_scope(lease, ScopeOutcome::Partial, 15).await?;
        partial(&store, &catalog, 20).await?;
        let sql = pool(&directory).await?;
        assert_eq!(frozen(&sql, &catalog).await?, before);
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_authorities WHERE id=?1")
                .bind(LINK)
                .fetch_one(&sql)
                .await?,
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_refreshes WHERE id=?1")
                .bind(LINK)
                .fetch_one(&sql)
                .await?,
            0
        );
        sql.close().await;
        assert!(store.projection_edges(None, 100).await?.edges.is_empty());
        if complete_before_misses {
            store
                .commit_page(
                    page(
                        &store,
                        &catalog,
                        "metadata",
                        "fresh-complete-before-misses",
                        25,
                        true,
                    )
                    .await?,
                )
                .await?;
            let sql = pool(&directory).await?;
            assert_eq!(frozen(&sql, &catalog).await?, before);
            assert_eq!(
                sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_authorities WHERE id=?1")
                    .bind(LINK)
                    .fetch_one(&sql)
                    .await?,
                1
            );
            assert_eq!(
                sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_refreshes WHERE id=?1")
                    .bind(LINK)
                    .fetch_one(&sql)
                    .await?,
                1
            );
            sql.close().await;
            assert!(store.projection_edges(None, 100).await?.edges.is_empty());
        }
        drop(store);
        let store = BinaryStore::open_with_catalog(&path, &bytes).await?;
        // An old missing_complete=1 is history: it cannot accelerate the first V3 miss.
        store
            .commit_page(page(&store, &catalog, "discover", "fresh-miss-one", 30, false).await?)
            .await?;
        let sql = pool(&directory).await?;
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT v3_misses FROM legacy_authorities WHERE id=?1")
                .bind(LINK)
                .fetch_one(&sql)
                .await?,
            1
        );
        assert_eq!(frozen(&sql, &catalog).await?, before);
        sql.close().await;
        assert_eq!(store.unattributed_authority_count().await?, 1);
        store
            .commit_page(page(&store, &catalog, "discover", "fresh-miss-two", 40, false).await?)
            .await?;
        if !complete_before_misses {
            assert_eq!(
                store.unattributed_authority_count().await?,
                1,
                "a buffered Partial facet is not certified by another stage's Complete"
            );
            assert!(store.projection_edges(None, 100).await?.edges.is_empty());
            store
                .commit_page(page(&store, &catalog, "metadata", "fresh-metadata", 50, true).await?)
                .await?;
        }
        assert_eq!(store.unattributed_authority_count().await?, 0);
        let edges = store.projection_edges(None, 100).await?.edges;
        assert_eq!(edges.len(), 1);
        let edge = edges.first().ok_or("fresh OAM edge absent")?;
        assert_eq!(
            (
                edge.from.id.as_str(),
                edge.to.account.as_str(),
                edge.to.id.as_str(),
                edge.relation.as_str()
            ),
            (LINK, "210987654321", SINK, "TARGETS_SINK")
        );
        drop(store);
        let reopened = BinaryStore::open_with_catalog(&path, &bytes).await?;
        assert_eq!(reopened.projection_edges(None, 100).await?.edges.len(), 1);
        let sql = pool(&directory).await?;
        assert_eq!(facet_row(&sql, true).await?, old_facet);
        sql.close().await;
    }
    Ok(())
}
