use super::*;
#[tokio::test]
async fn given_actual_oam_link_in_account_a_when_foreign_sink_arrives_in_either_order_then_only_observed_account_b_resolves()
-> TestResult {
    let bytes = oam_fixture::compiled(
        "oam",
        "2022-06-10",
        &["ListLinks", "ListSinks"],
        "AWS::Oam::Link",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for sink_first in [false, true] {
        let (_directory, store) = store(&catalog).await?;
        let operations = if sink_first {
            [("ListSinks", "210987654321"), ("ListLinks", "123456789012")]
        } else {
            [("ListLinks", "123456789012"), ("ListSinks", "210987654321")]
        };
        for (index, (operation, account)) in operations.into_iter().enumerate() {
            persist_oam(
                &store,
                &catalog,
                operation,
                account,
                i64::try_from(index)?
                    .checked_add(1)
                    .ok_or("time overflow")?,
            )
            .await?;
        }
        let resources = store.projection_resources(None, 100).await?;
        assert_eq!(resources.len(), 2);
        assert_eq!(
            resources
                .iter()
                .filter(|resource| resource.key.kind.as_str() == "AWS::Oam::Sink"
                    && resource.key.account.as_str() == "123456789012")
                .count(),
            0
        );
        let edges = store.projection_edges(None, 100).await?.edges;
        assert_eq!(edges.len(), 1);
        let edge = edges.first().ok_or("OAM sink edge absent")?;
        assert_eq!(edge.from.account.as_str(), "123456789012");
        assert_eq!(edge.from.id.as_str(), LINK);
        assert_eq!(edge.to.account.as_str(), "210987654321");
        assert_eq!(edge.to.id.as_str(), SINK);
        assert_eq!(edge.to.kind.as_str(), "AWS::Oam::Sink");
        assert_eq!(edge.relation.as_str(), "TARGETS_SINK");
    }
    Ok(())
}

#[tokio::test]
async fn given_reviewed_oam_page_when_public_store_receives_forged_relationships_then_every_transaction_is_rejected()
-> TestResult {
    use inventory_core::inventory::{Edge, EdgeSource, Kind, Relation};
    let bytes = oam_fixture::compiled(
        "oam",
        "2022-06-10",
        &["ListLinks", "ListSinks"],
        "AWS::Oam::Link",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for attack in [
        "unreviewed-same-account",
        "unreviewed-cross-account",
        "candidate-kind",
        "target-path",
        "target-value",
        "owner",
        "region",
        "absent-source",
        "direct-same-account",
        "direct-cross-account",
    ] {
        let (directory, store) = store(&catalog).await?;
        let mut page = oam_page(&store, &catalog, "ListLinks", "123456789012", 1).await?;
        let reference = page
            .unresolved
            .first_mut()
            .ok_or("reviewed OAM reference absent")?;
        match attack {
            "unreviewed-same-account" | "unreviewed-cross-account" => {
                reference.relation = Relation::parse("UNREVIEWED")?;
                if attack == "unreviewed-same-account" {
                    reference.target = ReferenceTarget::new(
                        AccountId::parse("123456789012")?,
                        reference.target.region().clone(),
                        reference.target.kind().clone(),
                        reference.target.path().to_vec(),
                        reference.target.value().clone(),
                    )?;
                }
            }
            "candidate-kind" => {
                reference.target = ReferenceTarget::new(
                    reference.target.account().clone(),
                    reference.target.region().clone(),
                    reference.target.kind().clone(),
                    reference.target.path().to_vec(),
                    reference.target.value().clone(),
                )?
                .with_candidate_kinds(vec![
                    reference.target.kind().clone(),
                    Kind::parse("AWS::Oam::Link")?,
                ])?;
            }
            "target-path" => {
                reference.target = ReferenceTarget::new(
                    reference.target.account().clone(),
                    reference.target.region().clone(),
                    reference.target.kind().clone(),
                    vec![FieldId::new(1)?, FieldId::new(1)?],
                    reference.target.value().clone(),
                )?;
            }
            "target-value" => {
                reference.target = ReferenceTarget::new(
                    reference.target.account().clone(),
                    reference.target.region().clone(),
                    reference.target.kind().clone(),
                    reference.target.path().to_vec(),
                    ResourceId::parse(&SINK.replace("000000000001", "000000000003"))?,
                )?;
            }
            "owner" => {
                reference.target = ReferenceTarget::new(
                    AccountId::parse("123456789012")?,
                    reference.target.region().clone(),
                    reference.target.kind().clone(),
                    reference.target.path().to_vec(),
                    reference.target.value().clone(),
                )?;
            }
            "region" => {
                reference.target = ReferenceTarget::new(
                    reference.target.account().clone(),
                    Region::parse("us-west-2")?,
                    reference.target.kind().clone(),
                    reference.target.path().to_vec(),
                    reference.target.value().clone(),
                )?;
            }
            "absent-source" => page.observations.clear(),
            "direct-same-account" | "direct-cross-account" => {
                let to = ResourceKey {
                    account: if attack == "direct-same-account" {
                        reference.from.account.clone()
                    } else {
                        reference.target.account().clone()
                    },
                    region: reference.target.region().clone(),
                    kind: reference.target.kind().clone(),
                    id: reference.target.value().clone(),
                };
                page.edges.push(Edge {
                    from: reference.from.clone(),
                    to,
                    relation: Relation::parse("UNREVIEWED")?,
                    source: EdgeSource::ReviewedRule,
                });
                page.unresolved.clear();
            }
            _ => return Err("unknown permission attack".into()),
        }
        let pool = sqlx::SqlitePool::connect(&format!(
            "sqlite://{}",
            directory.path().join("inventory.sqlite").display()
        ))
        .await?;
        type LeaseState = (i64, i64, i64, Option<Vec<u8>>, Vec<u8>, Vec<u8>);
        let before: LeaseState = sqlx::query_as(
            "SELECT state,generation,lease_until,token,bindings,context FROM work_queue",
        )
        .fetch_one(&pool)
        .await?;
        assert!(
            matches!(
                store.commit_page(page).await,
                Err(inventory_engine::inventory::StoreError::Invalid(
                    "relationship evidence is not permitted by the checked catalog"
                ))
            ),
            "{attack}"
        );
        assert!(
            store.projection_resources(None, 100).await?.is_empty(),
            "{attack}"
        );
        assert!(
            store.projection_edges(None, 100).await?.edges.is_empty(),
            "{attack}"
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs")
                .fetch_one(&pool)
                .await?,
            0,
            "{attack}"
        );
        let after: LeaseState = sqlx::query_as(
            "SELECT state,generation,lease_until,token,bindings,context FROM work_queue",
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(before, after, "{attack}");
        let extra: (i64, i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM resource_facets),(SELECT count(*) FROM projection_outbox),(SELECT count(*) FROM reviewed_reference_evidence),(SELECT count(*) FROM reviewed_reference_grants)").fetch_one(&pool).await?;
        assert_eq!(extra, (0, 0, 0, 0), "{attack}");
        pool.close().await;
    }
    Ok(())
}
