use super::*;
#[tokio::test]
async fn given_reviewed_direct_trust_edge_and_current_target_when_source_is_allow_or_deny_then_only_actual_allow_commits()
-> TestResult {
    use inventory_core::inventory::{Edge, EdgeSource};
    let catalog = fixture::catalog()?;
    for allow in [false, true] {
        let (_directory, store) = trust_store(&catalog).await?;
        // Both endpoints are the same actual observed IAM role; no endpoint is missing.
        let mut page = trust_page(
            &store,
            &catalog,
            ("123456789012", "collector"),
            &policy("Allow", SOURCE),
            1,
        )
        .await?;
        let reference = page
            .unresolved
            .first()
            .ok_or("self trust reference absent")?;
        page.edges.push(Edge {
            from: reference.from.clone(),
            to: reference.from.clone(),
            relation: reference.relation.clone(),
            source: EdgeSource::ReviewedRule,
        });
        page.unresolved.clear();
        if !allow {
            page.observations = trust_rows(
                &catalog,
                ("123456789012", "collector"),
                &policy("Deny", SOURCE),
            )?
            .1;
        }
        // A reviewed label alone cannot grant an edge, even with both real endpoints present.
        let result = store.commit_page(page).await;
        if allow {
            result?;
            assert_eq!(store.projection_resources(None, 100).await?.len(), 1);
            let edges = store.projection_edges(None, 100).await?.edges;
            assert_eq!(edges.len(), 1);
            let edge = edges.first().ok_or("self trust edge absent")?;
            assert_eq!(edge.from.id.as_str(), SOURCE);
            assert_eq!(edge.to.id.as_str(), SOURCE);
            assert_eq!(edge.relation.as_str(), "DECLARES_TRUST");
        } else {
            assert!(matches!(
                result,
                Err(StoreError::Invalid(
                    "relationship evidence is not permitted by the checked catalog"
                ))
            ));
            assert!(store.projection_resources(None, 100).await?.is_empty());
            assert!(store.projection_edges(None, 100).await?.edges.is_empty());
        }
    }
    Ok(())
}
