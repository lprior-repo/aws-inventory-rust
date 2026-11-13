//! successful empty parent discovery marks children complete without child calls behavior.
use super::*;

#[tokio::test]
async fn successful_empty_parent_discovery_marks_children_complete_without_child_calls()
-> TestResult {
    let (ctx, mut receiver) = context(3)?;
    let types = selected(&["AWS::EKS::Addon", "AWS::EKS::FargateProfile"])?;
    let group = one_group(&types)?;
    let calls = AtomicUsize::new(0);
    collect_group_with(
        &ctx,
        &group,
        |request| {
            calls.fetch_add(1, Ordering::Relaxed);
            async move {
                assert_eq!(request.type_name, "AWS::EKS::Cluster");
                assert!(request.resource_model.is_none());
                Ok(listed(&request.type_name, Vec::new(), None))
            }
        },
        |_| async { Err(ApiError::Permanent("unexpected get".into())) },
    )
    .await?;
    let events = events(&mut receiver)?;
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(events.len(), 3);
    assert!(
        events
            .iter()
            .all(|event| matches!(event, CollectionEvent::Page(_)))
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, CollectionEvent::Page(page)
        if page.scope.collector == "cloud_control_parent_discovery"))
            .count(),
        2
    );
    Ok(())
}

#[tokio::test]
async fn denied_parent_discovery_is_incomplete_and_never_claims_empty_success() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    collect_group_with(
        &ctx,
        &group,
        |_| async { Err(ApiError::Denied("fixture denied".into())) },
        |_| async { Err(ApiError::Permanent("unexpected get".into())) },
    )
    .await?;
    let events = events(&mut receiver)?;
    assert_eq!(events.len(), 2);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CollectionEvent::Failure(f)
        if f.category == "access_denied" && f.scope.collector == "cloud_control_list"))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CollectionEvent::Failure(f)
        if f.category == "parent_discovery_incomplete"))
    );
    assert!(
        events
            .iter()
            .all(|event| matches!(event, CollectionEvent::Failure(_)))
    );
    Ok(())
}

#[tokio::test]
async fn missing_parent_property_and_failed_parent_read_block_child_work() -> TestResult {
    for deny_get in [false, true] {
        let (ctx, mut receiver) = context(3)?;
        let types = selected(&["AWS::EKS::Addon"])?;
        let group = one_group(&types)?;
        let calls = AtomicUsize::new(0);
        collect_group_with(
            &ctx,
            &group,
            |request| {
                calls.fetch_add(1, Ordering::Relaxed);
                async move {
                    Ok(listed(
                        &request.type_name,
                        vec![description("alpha", json!({"Name":"alpha"}))],
                        None,
                    ))
                }
            },
            |request| async move {
                if deny_get {
                    Err(ApiError::Denied("parent detail denied".into()))
                } else {
                    Ok(got(
                        &request,
                        json!({"Arn":"arn:aws:eks:us-east-1:111111111111:cluster/alpha"}),
                    ))
                }
            },
        )
        .await?;
        let events = events(&mut receiver)?;
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, CollectionEvent::Failure(f)
            if f.category == "parent_discovery_incomplete"))
        );
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, CollectionEvent::Page(page)
            if page.scope.collector == "cloud_control_parent_discovery"))
        );
    }
    Ok(())
}

#[tokio::test]
async fn child_models_paginate_independently_and_parent_is_enumerated_once() -> TestResult {
    let (ctx, mut receiver) = context(3)?;
    let types = selected(&["AWS::EKS::Addon", "AWS::EKS::FargateProfile"])?;
    let group = one_group(&types)?;
    let requests = Mutex::new(Vec::new());
    collect_group_with(&ctx, &group, |request| {
        let recorded = requests.lock().map(|mut values| values.push(request.clone()))
            .map_err(|_| ApiError::Permanent("poisoned test requests".into()));
        async move {
            recorded?;
            if request.type_name == "AWS::EKS::Cluster" {
                return Ok(listed(&request.type_name, vec![description("alpha", json!({}))], None));
            }
            assert_eq!(request.resource_model.as_deref(), Some("{\"ClusterName\":\"alpha\"}"));
            match request.token.as_deref() {
                None => Ok(listed(&request.type_name, Vec::new(), Some("empty-but-more"))),
                Some("empty-but-more") => {
                    let id = if request.type_name == "AWS::EKS::Addon" { "alpha|coredns" } else { "alpha|serverless" };
                    Ok(listed(&request.type_name, vec![description(id, json!({"ClusterName":"alpha"}))], None))
                }
                _ => Err(ApiError::Permanent("unexpected page token".into())),
            }
        }
    }, |request| async move {
        let properties = match request.type_name.as_str() {
            "AWS::EKS::Cluster" => json!({"Name":"alpha","Arn":"arn:aws:eks:us-east-1:111111111111:cluster/alpha"}),
            "AWS::EKS::Addon" => json!({"ClusterName":"alpha","AddonName":"coredns","Arn":"arn:aws:eks:us-east-1:111111111111:addon/alpha/coredns/abc","ConfigurationValues":"HIDDEN"}),
            "AWS::EKS::FargateProfile" => json!({"ClusterName":"alpha","FargateProfileName":"serverless","Arn":"arn:aws:eks:us-east-1:111111111111:fargateprofile/alpha/serverless/abc","Subnets":["subnet-123"]}),
            _ => return Err(ApiError::Permanent("unexpected get type".into())),
        };
        Ok(got(&request,properties))
    }).await?;
    let requests = requests.into_inner().map_err(|_| "poisoned requests")?;
    assert_eq!(requests.len(), 5);
    assert_eq!(
        requests
            .iter()
            .filter(|r| r.type_name == "AWS::EKS::Cluster")
            .count(),
        1
    );
    let events = events(&mut receiver)?;
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, CollectionEvent::Failure(_)))
    );
    let resources: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            CollectionEvent::Page(page) => Some(&page.resources),
            _ => None,
        })
        .flatten()
        .collect();
    assert_eq!(resources.len(), 3);
    let child_edges: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            CollectionEvent::Page(page) => Some(&page.edges),
            _ => None,
        })
        .flatten()
        .filter(|edge| edge.relation == "CHILD_OF")
        .collect();
    assert_eq!(child_edges.len(), 2);
    assert!(
        child_edges
            .iter()
            .all(|edge| edge.to_id == "arn:aws:eks:us-east-1:111111111111:cluster/alpha")
    );
    assert!(!serde_json::to_string(&events)?.contains("HIDDEN"));
    Ok(())
}
