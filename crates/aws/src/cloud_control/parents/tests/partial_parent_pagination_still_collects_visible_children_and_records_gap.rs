//! partial parent pagination still collects visible children and records gap behavior.
use super::*;

#[tokio::test]
async fn partial_parent_pagination_still_collects_visible_children_and_records_gap() -> TestResult {
    let (ctx, mut receiver) = context(1)?;
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    collect_group_with(
        &ctx,
        &group,
        |request| async move {
            if request.type_name == "AWS::EKS::Cluster" {
                Ok(listed(
                    &request.type_name,
                    vec![description("alpha", json!({}))],
                    Some("more-parents"),
                ))
            } else {
                Ok(listed(&request.type_name, Vec::new(), None))
            }
        },
        |request| async move { Ok(got(&request, json!({"Name":"alpha"}))) },
    )
    .await?;
    let events = events(&mut receiver)?;
    assert!(
        events
            .iter()
            .any(|event| matches!(event,CollectionEvent::Failure(f) if f.category=="page_limit"))
    );
    assert!(events.iter().any(|event| matches!(event,CollectionEvent::Failure(f) if f.category=="parent_discovery_incomplete")));
    assert!(events.iter().any(|event| matches!(event,CollectionEvent::Page(page)
        if page.scope.collector=="cloud_control_list" && page.scope.parent_id.as_ref().is_some_and(|p|p.starts_with("AWS::EKS::Addon:")))));
    Ok(())
}

#[test]
fn mismatched_parent_is_rejected_and_thin_fallback_never_adds_parent_edges() -> TestResult {
    let (ctx, _receiver) = context(3)?;
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    let child = group.children.first().ok_or("missing child")?;
    let parent = parent_context(child, &fixture_resource(json!({"Name":"alpha"})))?;
    for provider in [
        json!({"ClusterName":"beta","AddonName":"coredns"}),
        json!({"AddonName":"coredns"}),
        json!({"ClusterName":1}),
    ] {
        let body = map_description(
            &ctx,
            child.descriptor,
            "alpha|coredns",
            &description("alpha|coredns", provider),
            true,
        )?;
        assert!(attach_parent(body, child, &parent, true).is_err());
    }
    let body = map_description(
        &ctx,
        child.descriptor,
        "alpha|coredns",
        &description("alpha|coredns", json!({})),
        false,
    )?;
    let fallback = attach_parent(body, child, &parent, false)?;
    assert!(fallback.edges.is_empty());
    assert_eq!(
        fallback
            .resources
            .first()
            .ok_or("missing fallback")?
            .properties["details_available"],
        false
    );
    Ok(())
}

#[tokio::test]
async fn cancellation_prevents_parent_and_child_calls() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    ctx.executor.cancel();
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    let calls = AtomicUsize::new(0);
    let outcome = collect_group_with(
        &ctx,
        &group,
        |_| {
            calls.fetch_add(1, Ordering::Relaxed);
            async { Err(ApiError::Permanent("unexpected list".into())) }
        },
        |_| async { Err(ApiError::Permanent("unexpected get".into())) },
    )
    .await;
    assert!(matches!(
        outcome,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert!(
        !events(&mut receiver)?
            .iter()
            .any(|event| matches!(event, CollectionEvent::Page(_)))
    );
    Ok(())
}

#[test]
fn model_and_child_budgets_fail_before_enrichment_and_cross_parent_duplicates_are_rejected()
-> TestResult {
    let ledger = Mutex::new(GroupLedger {
        models: (0..MAX_MODELS_PER_ROOT)
            .map(|index| <[u8; 32]>::from(Sha256::digest(index.to_le_bytes())))
            .collect(),
        items: BTreeSet::new(),
    });
    assert!(matches!(
        admit_model(&ledger, "new-scope"),
        Err(AwsScanError::ChildLimit)
    ));
    let descriptor = parent_catalog()?
        .into_iter()
        .next()
        .ok_or("missing child")?;
    let page = listed(
        &descriptor.type_name,
        vec![description("one", json!({}))],
        None,
    );
    admit_children(&ledger, &descriptor, &page)?;
    assert!(admit_children(&ledger, &descriptor, &page).is_err());
    let mut state = ledger.lock().map_err(|_| "poisoned ledger")?;
    state.models.pop_first();
    drop(state);
    admit_model(&ledger, "new-scope")?;
    assert!(matches!(
        admit_model(&ledger, "another-scope"),
        Err(AwsScanError::ChildLimit)
    ));
    assert!(matches!(
        admit_model(&ledger, "new-scope"),
        Err(AwsScanError::InvalidData(_))
    ));
    Ok(())
}

#[tokio::test]
async fn different_parents_cannot_restart_the_same_child_model_scope() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    let child_lists = AtomicUsize::new(0);
    collect_group_with(&ctx, &group, |request| {
        if request.resource_model.is_some() { child_lists.fetch_add(1, Ordering::Relaxed); }
        async move {
            if request.type_name == "AWS::EKS::Cluster" {
                Ok(listed(&request.type_name, vec![description("alpha", json!({})), description("beta", json!({}))], None))
            } else { Ok(listed(&request.type_name, Vec::new(), None)) }
        }
    }, |request| async move {
        Ok(got(&request, json!({"Name":"alpha", "Arn":format!("arn:aws:eks:us-east-1:111111111111:cluster/{}",request.identifier)})))
    }).await?;
    assert_eq!(child_lists.load(Ordering::Relaxed), 1);
    let events = events(&mut receiver)?;
    assert!(
        events
            .iter()
            .any(|event| matches!(event,CollectionEvent::Failure(f)
        if f.category == "invalid_response" && f.message.contains("same child list model")))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event,CollectionEvent::Failure(f)
        if f.category == "parent_discovery_incomplete"))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event,CollectionEvent::Page(page)
        if page.scope.collector == "cloud_control_parent_discovery"))
    );
    Ok(())
}
