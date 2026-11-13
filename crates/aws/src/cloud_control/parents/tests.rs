//! Literal parent models, adversarial scope outcomes, and deterministic properties.

use std::sync::atomic::AtomicUsize;

use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};
use tokio::sync::mpsc;

use super::*;

type TestError = Box<dyn std::error::Error>;
type TestResult = Result<(), TestError>;

fn context(max_pages: u32) -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
    let (sender, receiver) = mpsc::channel(512);
    Ok((
        Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages,
            page_size: 100,
        },
        receiver,
    ))
}

fn description(id: &str, properties: Value) -> ResourceDescription {
    ResourceDescription::builder()
        .identifier(id)
        .properties(properties.to_string())
        .build()
}

fn listed(
    type_name: &str,
    items: Vec<ResourceDescription>,
    token: Option<&str>,
) -> ListResourcesOutput {
    ListResourcesOutput::builder()
        .type_name(type_name)
        .set_resource_descriptions(Some(items))
        .set_next_token(token.map(str::to_owned))
        .build()
}

fn got(request: &GetRequest, properties: Value) -> GetResourceOutput {
    GetResourceOutput::builder()
        .type_name(&request.type_name)
        .resource_description(description(&request.identifier, properties))
        .build()
}

fn events(
    receiver: &mut mpsc::Receiver<CollectionEvent>,
) -> Result<Vec<CollectionEvent>, TestError> {
    let mut values = Vec::new();
    for _ in 0..512 {
        match receiver.try_recv() {
            Ok(value) => values.push(value),
            Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                return Ok(values);
            }
        }
    }
    Err("fixture event capacity exceeded".into())
}

fn selected(names: &[&str]) -> Result<Vec<ResourceType>, AwsScanError> {
    validate_selection(
        &names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
    )
}

fn one_group(types: &[ResourceType]) -> Result<RootGroup<'_>, TestError> {
    let mut groups = groups(types)?;
    if groups.len() != 1 {
        return Err("fixture must select one root group".into());
    }
    groups.pop().ok_or_else(|| "missing root group".into())
}

fn fixture_resource(properties: Value) -> Resource {
    Resource {
        id: "arn:aws:eks:us-east-1:111111111111:cluster/alpha".into(),
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        kind: "aws_eks_cluster".into(),
        name: Some("alpha".into()),
        properties: json!({"provider_properties":properties}),
        tags: BTreeMap::new(),
        source: Source::CloudControl,
        observed_at: 0,
    }
}

#[test]
fn combined_catalog_adds_exactly_twenty_one_reviewed_children() -> TestResult {
    let types = super::super::catalog()?;
    assert_eq!(types.len(), 418);
    assert_eq!(root_catalog()?.len(), 397);
    assert_eq!(parent_catalog()?.len(), 21);
    assert_eq!(
        types.iter().filter(|row| row.list_requires_model).count(),
        21
    );
    let groups = groups(&types)?;
    assert_eq!(groups.len(), 397);
    assert_eq!(
        groups
            .iter()
            .filter(|group| !group.children.is_empty())
            .count(),
        6
    );
    assert_eq!(
        groups
            .iter()
            .map(|group| group.children.len())
            .sum::<usize>(),
        21
    );
    assert!(validate_selection(&["AWS::ElasticLoadBalancingV2::ListenerRule".into()]).is_err());
    assert!(validate_selection(&["AWS::ECS::TaskSet".into()]).is_err());
    Ok(())
}

#[test]
fn child_only_selection_adds_each_prerequisite_once_and_retains_explicit_roots() -> TestResult {
    let types = selected(&[
        "AWS::EKS::Addon",
        "AWS::EKS::Nodegroup",
        "AWS::EKS::Addon",
        "AWS::EKS::Cluster",
    ])?;
    let names: BTreeSet<_> = types.iter().map(|row| row.type_name.as_str()).collect();
    assert_eq!(
        names,
        BTreeSet::from([
            "AWS::EKS::Cluster",
            "AWS::EKS::Addon",
            "AWS::EKS::Nodegroup"
        ])
    );
    assert_eq!(one_group(&types)?.children.len(), 2);
    assert_eq!(selected(&["AWS::EKS::Cluster"])?.len(), 1);
    assert_eq!(
        one_group(&selected(&["AWS::EKS::Cluster"])?)?
            .children
            .len(),
        0
    );
    let children = parent_catalog()?;
    assert!(groups(&children).is_err());
    Ok(())
}

#[test]
fn every_reviewed_binding_builds_the_literal_expected_model() -> TestResult {
    // Values and expected models are independent of the generated catalog.
    let fixtures = [
        (
            "AWS::EKS::Cluster",
            json!({"Name":"alpha"}),
            json!({"ClusterName":"alpha"}),
            3,
        ),
        (
            "AWS::ElasticLoadBalancingV2::LoadBalancer",
            json!({"LoadBalancerArn":"arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/api/0123456789abcdef"}),
            json!({"LoadBalancerArn":"arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/api/0123456789abcdef"}),
            1,
        ),
        (
            "AWS::ApiGateway::RestApi",
            json!({"RestApiId":"api123"}),
            json!({"RestApiId":"api123"}),
            8,
        ),
        (
            "AWS::ApiGatewayV2::Api",
            json!({"ApiId":"http123"}),
            json!({"ApiId":"http123"}),
            5,
        ),
        (
            "AWS::Logs::LogGroup",
            json!({"LogGroupName":"/aws/lambda/api"}),
            json!({"LogGroupName":"/aws/lambda/api"}),
            2,
        ),
        (
            "AWS::AppConfig::Application",
            json!({"ApplicationId":"app123"}),
            json!({"ApplicationId":"app123"}),
            2,
        ),
    ];
    let all = super::super::catalog()?;
    for (root, properties, expected, count) in fixtures {
        let group = groups(&all)?
            .into_iter()
            .find(|g| g.descriptor.type_name == root)
            .ok_or("missing expected root")?;
        assert_eq!(group.children.len(), count);
        let resource = fixture_resource(properties);
        for child in group.children {
            let context = parent_context(&child, &resource)?;
            assert_eq!(
                serde_json::from_str::<Value>(&context.encoded_model)?,
                expected
            );
            assert!(
                context
                    .scope_parent
                    .starts_with(&format!("{}:", child.descriptor.type_name))
            );
            assert_eq!(context.resource_id, resource.id);
            assert!(!context.encoded_model.contains("provider_properties"));
        }
    }
    Ok(())
}

#[test]
fn parent_context_rejects_missing_wrong_type_empty_control_and_oversized_values() -> TestResult {
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    let child = group.children.first().ok_or("missing child")?;
    for properties in [
        json!({}),
        json!({"Name":null}),
        json!({"Name":1}),
        json!({"Name":""}),
        json!({"Name":"bad\nname"}),
        json!({"Name":"x".repeat(1_025)}),
    ] {
        assert!(parent_context(child, &fixture_resource(properties)).is_err());
    }
    assert!(parent_context(child, &fixture_resource(json!({"Name":"x".repeat(1_024)}))).is_ok());
    Ok(())
}

#[test]
fn invalid_binding_cannot_reference_redacted_or_non_string_properties() -> TestResult {
    let types = selected(&["AWS::EKS::Addon"])?;
    let group = one_group(&types)?;
    let child = group.children.first().ok_or("missing child")?;
    let mut plan = child.plan.clone();
    for source in ["/DoesNotExist", "/Tags", "/Name/*"] {
        plan.bindings = vec![Binding {
            source_pointer: source.into(),
            model_property: "ClusterName".into(),
        }];
        assert!(validate_bindings(&plan, group.descriptor, child.descriptor).is_err());
    }
    for target in [
        "ConfigurationValues",
        "Tags",
        "Name/Injected",
        "",
        "Cluster_Name",
    ] {
        plan.bindings = vec![Binding {
            source_pointer: "/Name".into(),
            model_property: target.into(),
        }];
        assert!(validate_bindings(&plan, group.descriptor, child.descriptor).is_err());
    }
    Ok(())
}

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

#[test]
fn literal_emulator_parent_fixtures_preserve_all_sanitized_properties_and_exact_architecture_edges()
-> TestResult {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/cloudcontrol-parents.json"
    ))?;
    let (mut ctx, _receiver) = context(3)?;
    ctx.account = "000000000000".into();
    let catalog = super::super::catalog()?;
    let plans = plans()?;
    let parents = fixture
        .get("parents")
        .and_then(Value::as_array)
        .ok_or("missing literal parents")?;
    let children = fixture
        .get("children")
        .and_then(Value::as_array)
        .ok_or("missing literal children")?;
    let pagination = fixture
        .get("pagination_children")
        .and_then(Value::as_array)
        .ok_or("missing pagination children")?;
    assert_eq!(children.len(), 63);
    assert_eq!(pagination.len(), 98);
    let text = |row: &Value, key: &str| -> Result<String, TestError> {
        row.get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("missing fixture field {key}").into())
    };
    let mut architecture_edges = 0_usize;
    for row in children.iter().chain(pagination) {
        let type_name = text(row, "type_name")?;
        let descriptor = find_type(&catalog, &type_name)?;
        let plan = plans
            .iter()
            .find(|plan| plan.type_name == type_name)
            .ok_or("missing fixture plan")?
            .clone();
        let child = ChildType { descriptor, plan };
        let parent_key = text(row, "parent_key")?;
        let parent_fixture = parents
            .iter()
            .find(|parent| parent.get("key").and_then(Value::as_str) == Some(parent_key.as_str()))
            .ok_or("missing fixture parent")?;
        let parent_id = parent_fixture
            .get("expected_resource_id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("literal-parent:{parent_key}"));
        let parent_resource = Resource {
            id: parent_id.clone(),
            kind: text(parent_fixture, "expected_kind")?,
            account_id: ctx.account.clone(),
            region: ctx.region.clone(),
            name: None,
            tags: BTreeMap::new(),
            source: Source::CloudControl,
            observed_at: 10,
            properties: json!({"provider_properties":parent_fixture.get("provider").ok_or("missing parent provider")?}),
        };
        let parent = parent_context(&child, &parent_resource)?;
        assert_eq!(
            serde_json::to_value(&parent.model)?,
            *row.get("expected_model").ok_or("missing expected model")?
        );
        let id = text(row, "name")?;
        let provider = row.get("provider").ok_or("missing child provider")?.clone();
        let body = map_description(&ctx, descriptor, &id, &description(&id, provider), true)?;
        let body = attach_parent(body, &child, &parent, true)?;
        let resource = body.resources.first().ok_or("missing mapped child")?;
        assert_eq!(resource.kind, text(row, "expected_kind")?);
        assert_eq!(
            resource.properties.get("provider_properties"),
            row.get("expected_provider")
        );
        assert!(!serde_json::to_string(resource)?.contains("HIDDEN"));
        let expected = row
            .get("expected_relationships")
            .and_then(Value::as_array)
            .ok_or("missing expected relationships")?;
        assert_eq!(body.edges.len(), expected.len() + 1);
        let parent_edges: Vec<_> = body
            .edges
            .iter()
            .filter(|edge| edge.relation == "CHILD_OF")
            .collect();
        assert_eq!(parent_edges.len(), 1);
        assert_eq!(
            parent_edges.first().ok_or("missing parent edge")?.to_id,
            parent_id
        );
        for wanted in expected {
            let relation = text(wanted, "relation")?;
            let target = text(wanted, "target")?;
            let matches: Vec<_> = body
                .edges
                .iter()
                .filter(|edge| edge.relation == relation && edge.to_id == target)
                .collect();
            assert_eq!(matches.len(), 1, "{type_name}: {relation} -> {target}");
            let edge = matches.first().ok_or("missing architecture edge")?;
            assert_eq!(edge.properties.get("target_kind"), wanted.get("kind"));
            assert_eq!(edge.properties.get("property_paths"), wanted.get("paths"));
            assert_eq!(
                edge.properties.get("owner_resolution"),
                wanted.get("owner_resolution")
            );
            architecture_edges += 1;
        }
    }
    assert_eq!(architecture_edges, 24);
    Ok(())
}

proptest! {
    #![proptest_config(Config { cases: 128, rng_seed: RngSeed::Fixed(0x06cc_21), ..Config::default() })]

    #[test]
    fn property_parent_models_are_escaped_exactly_and_scope_identity_is_value_sensitive(
        name in "[A-Za-z0-9_./\\\"\\\\-]{1,64}", suffix in "[a-z]{1,8}",
    ) {
        let types=selected(&["AWS::EKS::Addon"]).map_err(|e|TestCaseError::fail(e.to_string()))?;
        let group=one_group(&types).map_err(|e|TestCaseError::fail(e.to_string()))?;
        let child=group.children.first().ok_or_else(||TestCaseError::fail("missing child"))?;
        let context=parent_context(child,&fixture_resource(json!({"Name":name}))).map_err(|e|TestCaseError::fail(e.to_string()))?;
        let decoded: Value=serde_json::from_str(&context.encoded_model).map_err(|e|TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(decoded,json!({"ClusterName":name}));
        let different=parent_context(child,&fixture_resource(json!({"Name":format!("{name}{suffix}")}))).map_err(|e|TestCaseError::fail(e.to_string()))?;
        prop_assert_ne!(context.scope_parent,different.scope_parent);
    }

    #[test]
    fn property_selection_is_idempotent_and_closed_under_prerequisites(indices in prop::collection::vec(0_usize..21,1..32)) {
        let children=parent_catalog().map_err(|e|TestCaseError::fail(e.to_string()))?;
        let requested:Vec<_>=indices.iter().filter_map(|index|children.get(*index).map(|row|row.type_name.clone())).collect();
        let first=validate_selection(&requested).map_err(|e|TestCaseError::fail(e.to_string()))?;
        let first_names:Vec<_>=first.iter().map(|row|row.type_name.clone()).collect();
        let second=validate_selection(&first_names).map_err(|e|TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(&first_names,&second.iter().map(|row|row.type_name.clone()).collect::<Vec<_>>());
        let planned=groups(&first).map_err(|e|TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(planned.iter().map(|group|group.children.len()).sum::<usize>(),requested.iter().collect::<BTreeSet<_>>().len());
    }
}
