//! child denial and child read fallback remain failed without parent edges behavior.
use super::*;

#[tokio::test]
async fn child_denial_and_child_read_fallback_remain_failed_without_parent_edges() -> TestResult {
    for deny_list in [false, true] {
        let (ctx, mut receiver) = context(3)?;
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
                        None,
                    ))
                } else if deny_list {
                    Err(ApiError::Denied("child list denied".into()))
                } else {
                    Ok(listed(
                        &request.type_name,
                        vec![description(
                            "alpha|coredns",
                            json!({"ClusterName":"alpha","AddonName":"coredns"}),
                        )],
                        None,
                    ))
                }
            },
            |request| async move {
                if request.type_name == "AWS::EKS::Cluster" {
                    Ok(got(&request, json!({"Name":"alpha"})))
                } else {
                    Err(ApiError::Denied("child read denied".into()))
                }
            },
        )
        .await?;
        let events = events(&mut receiver)?;
        assert_eq!(events.iter().filter(|event| matches!(event,CollectionEvent::Failure(f) if f.category=="access_denied")).count(),1);
        assert!(
            events
                .iter()
                .any(|event| matches!(event,CollectionEvent::Page(page)
            if page.scope.collector=="cloud_control_parent_discovery"))
        );
        let child_resources: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                CollectionEvent::Page(page) => Some(&page.resources),
                _ => None,
            })
            .flatten()
            .filter(|resource| {
                resource.properties.get("cloudformation_type") == Some(&json!("AWS::EKS::Addon"))
            })
            .collect();
        assert_eq!(child_resources.len(), if deny_list { 0 } else { 1 });
        if let Some(resource) = child_resources.first() {
            assert_eq!(
                resource.properties.get("details_available"),
                Some(&Value::Bool(false))
            );
        }
        assert!(
            !events
                .iter()
                .any(|event| matches!(event,CollectionEvent::Page(page) if !page.edges.is_empty()))
        );
    }
    Ok(())
}

#[tokio::test]
async fn cancellation_after_child_admission_stops_child_reads_and_discovery_completion()
-> TestResult {
    let (ctx, mut receiver) = context(3)?;
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    let child_reads = AtomicUsize::new(0);
    let result = collect_group_with(
        &ctx,
        &group,
        |request| {
            if request.resource_model.is_some() {
                ctx.executor.cancel();
            }
            async move {
                if request.type_name == "AWS::EKS::Cluster" {
                    Ok(listed(
                        &request.type_name,
                        vec![description("alpha", json!({}))],
                        None,
                    ))
                } else {
                    Ok(listed(
                        &request.type_name,
                        vec![description("alpha|coredns", json!({"ClusterName":"alpha"}))],
                        None,
                    ))
                }
            }
        },
        |request| {
            if request.type_name == "AWS::EKS::Addon" {
                child_reads.fetch_add(1, Ordering::Relaxed);
            }
            async move { Ok(got(&request, json!({"Name":"alpha"}))) }
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(AwsScanError::Api(ApiError::Cancelled))
    ));
    assert_eq!(child_reads.load(Ordering::Relaxed), 0);
    assert!(
        !events(&mut receiver)?
            .iter()
            .any(|event| matches!(event,CollectionEvent::Page(page)
        if page.scope.collector=="cloud_control_parent_discovery"))
    );
    Ok(())
}

#[test]
fn aggregate_child_capacity_accepts_the_last_slot_then_rejects_without_mutation() -> TestResult {
    let items = (0..MAX_CHILDREN.saturating_sub(1))
        .map(|index| <[u8; 32]>::from(Sha256::digest(index.to_le_bytes())))
        .collect();
    let ledger = Mutex::new(GroupLedger {
        models: BTreeSet::new(),
        items,
    });
    let descriptor = parent_catalog()?
        .into_iter()
        .next()
        .ok_or("missing child")?;
    let last = listed(
        &descriptor.type_name,
        vec![description("last-available-slot", json!({}))],
        None,
    );
    admit_children(&ledger, &descriptor, &last)?;
    assert_eq!(
        ledger.lock().map_err(|_| "poisoned ledger")?.items.len(),
        MAX_CHILDREN
    );
    let extra = listed(
        &descriptor.type_name,
        vec![
            description("overflow-one", json!({})),
            description("overflow-two", json!({})),
        ],
        None,
    );
    assert!(matches!(
        admit_children(&ledger, &descriptor, &extra),
        Err(AwsScanError::ChildLimit)
    ));
    assert_eq!(
        ledger.lock().map_err(|_| "poisoned ledger")?.items.len(),
        MAX_CHILDREN
    );
    Ok(())
}
