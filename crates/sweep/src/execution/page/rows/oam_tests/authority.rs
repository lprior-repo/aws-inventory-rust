use super::*;
#[tokio::test]
async fn given_legacy_matching_foreign_reference_when_catalog_is_attached_then_only_fresh_source_proof_can_promote_it()
-> TestResult {
    let bytes = oam_fixture::compiled(
        "oam",
        "2022-06-10",
        &["ListLinks", "ListSinks"],
        "AWS::Oam::Link",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for version in [14_i64, 15] {
        let (directory, original) = store(&catalog).await?;
        // Given an old real source whose payload declares a DIFFERENT foreign sink.
        let mut page = oam_page(&original, &catalog, "ListLinks", "123456789012", 1).await?;
        let plan = catalog
            .lookup("oam", "ListLinks")
            .ok_or("OAM plan absent")?;
        let other = SINK.replace("000000000001", "000000000003");
        let rows = oam_fixture::rows(plan, json!({"Items":[{"Arn":LINK,"SinkArn":other}]}))?;
        let account = AccountId::parse("123456789012")?;
        let home = RegionId::parse("us-east-1")?;
        let request = Record::encode(&Value::Map(BTreeMap::new()))?;
        page.observations = rows
            .iter()
            .map(|row| {
                observation(
                    &catalog,
                    record::IdentityContext {
                        account: &account,
                        region: &home,
                    },
                    plan,
                    &Region::Regional(home.clone()),
                    row.clone(),
                    &request,
                    &DiscoveryContext::default(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        page.unresolved = references(
            &catalog,
            plan,
            &home,
            &rows,
            &page.observations,
            &request,
            None,
        )?;
        original.commit_page(page).await?;
        drop(original);
        let path = directory.path().join("inventory.sqlite");
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        let mut tx = pool.begin().await?;
        // A historical store predates the derived candidate index.
        sqlx::query("DROP TABLE unresolved_ref_candidates")
            .execute(&mut *tx)
            .await?;
        // This historical unchecked tuple matches a reviewed rule but not its actual source value.
        sqlx::query("UPDATE unresolved_refs SET target_value=?1")
            .bind(SINK)
            .execute(&mut *tx)
            .await?;
        downgrade_oam(&mut tx, version).await?;
        tx.commit().await?;
        pool.close().await;
        // When catalog attachment is followed ONLY by an unrelated foreign target arrival.
        let reopened = BinaryStore::open_with_catalog(&path, &bytes).await?;
        persist_oam(&reopened, &catalog, "ListSinks", "210987654321", 2).await?;
        assert_eq!(reopened.projection_resources(None, 100).await?.len(), 2);
        assert!(
            reopened.projection_edges(None, 100).await?.edges.is_empty(),
            "schema {version}"
        );
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs WHERE from_id=?1")
                .bind(LINK)
                .fetch_one(&pool)
                .await?,
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM reviewed_reference_evidence")
                .fetch_one(&pool)
                .await?,
            0
        );
        pool.close().await;
        // Then history stays dormant, but a later genuine source page can prove the same tuple.
        persist_oam(&reopened, &catalog, "ListLinks", "123456789012", 3).await?;
        let edges = reopened.projection_edges(None, 100).await?.edges;
        assert_eq!(edges.len(), 1, "schema {version}");
        let edge = edges.first().ok_or("fresh reviewed OAM edge absent")?;
        assert_eq!(edge.from.id.as_str(), LINK);
        assert_eq!(edge.to.id.as_str(), SINK);
        assert_eq!(edge.to.account.as_str(), "210987654321");
        assert_eq!(edge.relation.as_str(), "TARGETS_SINK");
    }
    Ok(())
}

#[tokio::test]
async fn given_reviewed_oam_owner_when_source_partition_or_concrete_target_grammar_is_forged_then_public_store_rejects_atomically()
-> TestResult {
    let bytes = oam_fixture::compiled(
        "oam",
        "2022-06-10",
        &["ListLinks", "ListSinks"],
        "AWS::Oam::Link",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for attack in ["aws-us-gov", "aws-cn", "wildcard-many", "wildcard-one"] {
        let (directory, store) = store(&catalog).await?;
        let home = match attack {
            "aws-us-gov" => "us-gov-west-1",
            "aws-cn" => "cn-north-1",
            _ => "us-east-1",
        };
        let link = LINK.replace("us-east-1", home);
        let link = if attack.starts_with("aws-") {
            link.replacen("arn:aws:", &format!("arn:{attack}:"), 1)
        } else {
            link
        };
        let sink = SINK.replace("us-east-1", home);
        let mut page = oam_page_in_scope(
            &store,
            &catalog,
            "ListLinks",
            "123456789012",
            1,
            (home, &link, &sink),
        )
        .await?;
        if attack.starts_with("aws-") {
            assert!(page.unresolved.is_empty(), "{attack}");
            let plan = catalog
                .lookup("oam", "ListLinks")
                .ok_or("OAM plan absent")?;
            let east = RegionId::parse("us-east-1")?;
            let account = AccountId::parse("123456789012")?;
            let request = Record::encode(&Value::Map(BTreeMap::new()))?;
            let rows = oam_fixture::rows(plan, json!({"Items":[{"Arn":LINK,"SinkArn":SINK}]}))?;
            let observations = rows
                .iter()
                .map(|row| {
                    observation(
                        &catalog,
                        record::IdentityContext {
                            account: &account,
                            region: &east,
                        },
                        plan,
                        &Region::Regional(east.clone()),
                        row.clone(),
                        &request,
                        &DiscoveryContext::default(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut reviewed =
                references(&catalog, plan, &east, &rows, &observations, &request, None)?;
            let mut forged = reviewed.pop().ok_or("reviewed OAM reference absent")?;
            forged.from = page
                .observations
                .first()
                .ok_or("forged source observation absent")?
                .key()
                .clone();
            forged.target = ReferenceTarget::typed(
                forged.target.account().clone(),
                Region::Regional(RegionId::parse(home)?),
                forged.target.kind().clone(),
                forged.target.path().to_vec(),
                ResourceId::parse(&sink)?,
                forged.target.mode(),
            )?;
            page.unresolved.push(forged);
        }
        if attack.starts_with("wildcard-") {
            let suffix = if attack == "wildcard-many" {
                "*"
            } else {
                "targe?"
            };
            let forged = format!("arn:aws:oam:{home}:210987654321:sink/{suffix}");
            let plan = catalog
                .lookup("oam", "ListLinks")
                .ok_or("OAM plan absent")?;
            let rows = oam_fixture::rows(plan, json!({"Items":[{"Arn":link,"SinkArn":forged}]}))?;
            let account = AccountId::parse("123456789012")?;
            let home = RegionId::parse(home)?;
            let request = Record::encode(&Value::Map(BTreeMap::new()))?;
            page.observations = rows
                .iter()
                .map(|row| {
                    observation(
                        &catalog,
                        record::IdentityContext {
                            account: &account,
                            region: &home,
                        },
                        plan,
                        &Region::Regional(home.clone()),
                        row.clone(),
                        &request,
                        &DiscoveryContext::default(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let reference = page.unresolved.first_mut().ok_or("OAM reference absent")?;
            reference.target = ReferenceTarget::new(
                reference.target.account().clone(),
                reference.target.region().clone(),
                reference.target.kind().clone(),
                reference.target.path().to_vec(),
                ResourceId::parse(&forged)?,
            )?;
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
        let after: LeaseState = sqlx::query_as(
            "SELECT state,generation,lease_until,token,bindings,context FROM work_queue",
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(before, after, "{attack}");
        let persisted: (i64, i64, i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM unresolved_refs),(SELECT count(*) FROM resource_facets),(SELECT count(*) FROM projection_outbox),(SELECT count(*) FROM reviewed_reference_evidence),(SELECT count(*) FROM reviewed_reference_grants)").fetch_one(&pool).await?;
        assert_eq!(persisted, (0, 0, 0, 0, 0), "{attack}");
        pool.close().await;
    }
    Ok(())
}
