use super::*;
#[tokio::test]
async fn given_five_native_constraint_types_in_one_page_when_dispatched_then_each_exact_canonical_row_and_portfolio_edge_persists()
-> TestResult {
    let catalog = catalog()?;
    let plan = catalog
        .lookup(SERVICE, OPERATION)
        .ok_or("constraint plan absent")?;
    let (root, store, lease) = flow::rooted(&catalog).await?;
    let sweep = lease.sweep().clone();
    let details = MAPPINGS.iter().enumerate().map(|(index, (value, _))| json!({
        "ConstraintId": format!("cons-{index}"), "Type": value, "Description": format!("metadata-{index}"),
    })).collect::<Vec<_>>();
    let body = json!({"ConstraintDetails":details});
    let native = flow::decode(plan, body.clone())?;
    let rows = record::item_records(&native.record, plan.items())?;
    let parts = flow::prepare(&catalog, &lease, body).await?;
    assert_eq!(parts.observations.len(), 5);
    assert_eq!(parts.references.len(), 5);
    assert!(parts.children.is_empty());
    flow::commit(&store, lease, parts).await?;
    let store = flow::reopen(store, &root, &catalog).await?;
    let reader = flow::reader(&root).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE kind LIKE 'AWS::ServiceCatalog::%Constraint'").fetch_one(&reader).await?, 5);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets")
            .fetch_one(&reader)
            .await?,
        6
    );
    for (index, ((_, kind), row)) in MAPPINGS.iter().zip(&rows).enumerate() {
        let id = format!("8:port-one|6:cons-{index}");
        let schema = catalog
            .schemas()
            .iter()
            .find(|schema| schema.kind().as_str() == *kind)
            .ok_or("schema absent")?;
        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT id,record FROM \"");
        query.push(schema.table().as_str()).push("\"");
        let saved: Vec<(String, Vec<u8>)> = query.build_query_as().fetch_all(&reader).await?;
        assert_eq!(saved, [(id.clone(), row.as_bytes().to_vec())], "{kind}");
        let key = ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: Region::parse("us-east-1")?,
            kind: inventory_core::inventory::Kind::parse(kind)?,
            id: ResourceId::parse(&id)?,
        };
        let edges = store.projection_edges_for(&key, None, 10).await?;
        assert_eq!(edges.edges.len(), 1, "{kind}");
        let edge = edges.edges.first().ok_or("portfolio edge absent")?;
        assert_eq!(edge.from, key);
        assert_eq!(edge.relation.as_str(), "FOR_PORTFOLIO");
        assert_eq!(edge.to.kind.as_str(), "AWS::ServiceCatalog::Portfolio");
        assert_eq!(edge.to.id.as_str(), PORTFOLIO_ARN);
        assert_eq!(edge.to.account, AccountId::parse("123456789012")?);
        assert_eq!(edge.to.region, Region::parse("us-east-1")?);
    }
    let summary = store.sweep_summary(&sweep).await?;
    assert_eq!(summary.complete_scopes, 2);
    assert_eq!(summary.work_pending, 0);
    reader.close().await;
    Ok(())
}

#[tokio::test]
async fn given_unknown_constraint_after_known_row_when_dispatched_then_incomplete_coverage_is_explicit_and_no_generic_resource_persists()
-> TestResult {
    let catalog = catalog()?;
    let (root, store, lease) = flow::rooted(&catalog).await?;
    let error = flow::prepare(&catalog, &lease, json!({"ConstraintDetails":[
        {"ConstraintId":"known","Type":"LAUNCH"}, {"ConstraintId":"future","Type":"FUTURE_CONSTRAINT"}
    ]})).await.err().ok_or("unknown discriminator unexpectedly assembled")?;
    assert_eq!(
        error.to_string(),
        "runtime catalog error: incomplete resource kind coverage: unknown discriminator \"FUTURE_CONSTRAINT\""
    );
    assert!(matches!(error.downcast_ref::<SweepError>(),
        Some(SweepError::Catalog(RuntimeCatalogError::UnclassifiedKind { discriminator })) if discriminator == "FUTURE_CONSTRAINT"));
    let _store = flow::reopen(store, &root, &catalog).await?;
    let reader = flow::reader(&root).await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets")
            .fetch_one(&reader)
            .await?,
        1
    );
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE kind LIKE 'AWS::ServiceCatalog::%Constraint'").fetch_one(&reader).await?, 0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs")
            .fetch_one(&reader)
            .await?,
        0
    );
    reader.close().await;
    Ok(())
}

#[test]
fn given_missing_null_or_nontext_typed_discriminator_when_dispatched_then_malformed_input_is_not_unknown_coverage()
-> TestResult {
    let catalog = catalog()?;
    let plan = catalog
        .lookup(SERVICE, OPERATION)
        .ok_or("constraint plan absent")?;
    for value in [
        None,
        Some(Value::Null),
        Some(Value::Bool(true)),
        Some(Value::Int(1)),
    ] {
        let mut fields = BTreeMap::from([(
            Key::Field(field("ConstraintId")?),
            Value::Text("cons-one".to_owned()),
        )]);
        if let Some(value) = value {
            fields.insert(Key::Field(field("Type")?), value);
        }
        let row = Record::encode(&Value::Map(fields))?;
        let result = dispatched(&catalog, plan, &[row]);
        assert!(matches!(
            result,
            Err(SweepError::Catalog(RuntimeCatalogError::Capability))
        ));
    }
    Ok(())
}

#[tokio::test]
async fn given_each_classified_kind_with_wrong_request_evidence_when_committed_then_checked_authorization_rejects_atomically()
-> TestResult {
    let catalog = catalog()?;
    for (value, _) in MAPPINGS {
        let (root, store, lease) = flow::rooted(&catalog).await?;
        let mut parts = flow::prepare(
            &catalog,
            &lease,
            json!({"ConstraintDetails":[{"ConstraintId":"cons-one","Type":value}]}),
        )
        .await?;
        assert_eq!(parts.references.len(), 1, "{value}");
        let reference = parts
            .references
            .first_mut()
            .ok_or("assembled portfolio reference absent")?;
        assert_eq!(reference.target.value().as_str(), PORTFOLIO);
        reference.target = inventory_core::inventory::ReferenceTarget::new(
            AccountId::parse("123456789012")?,
            Region::parse("us-east-1")?,
            inventory_core::inventory::Kind::parse("AWS::ServiceCatalog::Portfolio")?,
            vec![field("Id")?],
            ResourceId::parse("port-other")?,
        )?;
        let error = flow::commit(&store, lease, parts)
            .await
            .err()
            .ok_or("forged request evidence committed")?;
        assert!(
            matches!(
                error.downcast_ref::<inventory_engine::inventory::StoreError>(),
                Some(inventory_engine::inventory::StoreError::Invalid(
                    "relationship evidence is not permitted by the checked catalog"
                ))
            ),
            "{value}"
        );
        let _store = flow::reopen(store, &root, &catalog).await?;
        let reader = flow::reader(&root).await?;
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets")
                .fetch_one(&reader)
                .await?,
            1
        );
        assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE kind LIKE 'AWS::ServiceCatalog::%Constraint'").fetch_one(&reader).await?, 0);
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs")
                .fetch_one(&reader)
                .await?,
            0
        );
        reader.close().await;
    }
    Ok(())
}
