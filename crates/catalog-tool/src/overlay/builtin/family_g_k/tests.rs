pub(crate) mod contracts;

use crate::classes::DenyReason;
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, MemberPath, OperationName, OperationOverlay,
    OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const FAMILIES: &[&str] = &[
    "gamelift",
    "gameliftstreams",
    "geo-maps",
    "geo-places",
    "geo-routes",
    "glacier",
    "global-accelerator",
    "glue",
    "grafana",
    "greengrass",
    "greengrassv2",
    "groundstation",
    "guardduty",
    "health",
    "healthlake",
    "iam-toolbox",
    "identitystore",
    "imagebuilder",
    "inspector",
    "inspector-scan",
    "inspector2",
    "interconnect",
    "internetmonitor",
    "invoicing",
    "iot",
    "iot-data-plane",
    "iot-jobs-data-plane",
    "iot-managed-integrations",
    "iot-wireless",
    "iotdeviceadvisor",
    "iotfleetwise",
    "iotsecuretunneling",
    "iotsitewise",
    "iotthingsgraph",
    "iottwinmaker",
    "ivs",
    "ivs-realtime",
    "ivschat",
    "kafka",
    "kafkaconnect",
    "kendra",
    "kendra-ranking",
    "keyspaces",
    "keyspacesstreams",
    "kinesis",
    "kinesis-analytics",
    "kinesis-analytics-v2",
    "kinesis-video",
    "kinesis-video-archived-media",
    "kinesis-video-media",
    "kinesis-video-signaling",
    "kinesis-video-webrtc-storage",
];

fn operation<'a>(
    policies: &'a [ServiceOverlay],
    service: &str,
    operation: &str,
) -> Result<&'a OperationOverlay, Box<dyn std::error::Error>> {
    let policy = policies
        .iter()
        .find(|policy| policy.service() == service)
        .ok_or_else(|| format!("missing service {service}"))?;
    policy
        .operations()
        .get(&OperationName::new(operation)?)
        .ok_or_else(|| format!("missing operation {service}/{operation}").into())
}

#[test]
fn embedded_catalog_exposes_every_reviewed_g_k_operation_with_explicit_disposition() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let family = policies
        .into_iter()
        .filter(|policy| FAMILIES.contains(&policy.service()))
        .collect::<Vec<_>>();
    assert_eq!(
        family
            .iter()
            .map(ServiceOverlay::service)
            .collect::<std::collections::BTreeSet<_>>(),
        FAMILIES.iter().copied().collect()
    );
    contracts::assert_reviewed(&family)?;
    Ok(())
}

#[test]
fn credential_content_and_billed_calls_remain_denied_without_projection() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let cases = [
        ("gamelift", "GetComputeAccess", DenyReason::Secret),
        ("glue", "GetJob", DenyReason::Secret),
        ("grafana", "ListWorkspaces", DenyReason::Secret),
        ("iot-wireless", "GetWirelessDevice", DenyReason::Secret),
        ("ivs", "GetChannel", DenyReason::Secret),
        ("ivs-realtime", "GetIngestConfiguration", DenyReason::Secret),
        (
            "kafkaconnect",
            "DescribeWorkerConfiguration",
            DenyReason::Secret,
        ),
        ("glacier", "GetJobOutput", DenyReason::Content),
        (
            "imagebuilder",
            "ListImageBuildVersions",
            DenyReason::Content,
        ),
        ("kinesis", "GetRecords", DenyReason::Content),
        ("iot", "ListThings", DenyReason::Paid),
        ("geo-routes", "CalculateRoutes", DenyReason::Paid),
        ("iot", "DescribeEndpoint", DenyReason::Mutating),
    ];
    for (service, name, reason) in cases {
        let decision = operation(&policies, service, name)?;
        assert_eq!(
            decision.class,
            Some(OverlayClass::Deny(reason)),
            "{service}/{name}"
        );
        assert_eq!(decision.identity, None);
        assert_eq!(decision.items, None);
        assert_eq!(decision.paginator, None);
    }
    Ok(())
}

#[test]
fn sitewise_collects_all_model_scoped_assets_instead_of_only_top_level_assets() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let decision = operation(&policies, "iotsitewise", "ListAssets")?;
    assert_eq!(decision.class, Some(OverlayClass::Expand));
    assert_eq!(
        decision.bind.get(&MemberPath::selected("assetModelId")?),
        Some(&Binding::From {
            operation: OperationName::new("ListAssetModels")?,
            path: MemberPath::selected("id")?,
        })
    );
    assert_eq!(
        decision.fixed.get(&MemberPath::selected("filter")?),
        Some(&FixedValue::String("ALL".to_owned()))
    );
    assert_eq!(
        decision.paginator,
        Some(Paginator::Token(TokenPair {
            input: MemberPath::selected("nextToken")?,
            output: MemberPath::selected("nextToken")?,
            optional: false,
        }))
    );
    Ok(())
}

#[test]
fn keyspaces_details_bind_both_table_identifiers_from_one_enumerated_parent() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let decision = operation(&policies, "keyspaces", "GetTable")?;
    assert_eq!(decision.class, Some(OverlayClass::Expand));
    for member in ["keyspaceName", "tableName"] {
        assert_eq!(
            decision.bind.get(&MemberPath::selected(member)?),
            Some(&Binding::From {
                operation: OperationName::new("ListTables")?,
                path: MemberPath::selected(member)?,
            })
        );
    }
    assert_eq!(
        decision.identity,
        Some(Identity::Arn(MemberPath::selected("resourceArn")?))
    );
    assert_eq!(decision.paginator, Some(Paginator::None));
    Ok(())
}

#[test]
fn scalar_resource_ids_use_the_whole_selected_item_and_real_cursor_case() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let decision = operation(&policies, "glue", "ListJobs")?;
    assert_eq!(
        decision.items,
        Some(ItemSelector::Member(MemberPath::selected("JobNames")?))
    );
    assert_eq!(
        decision.identity,
        Some(Identity::Key(vec![MemberPath::selected("$")?]))
    );
    assert_eq!(
        decision.paginator,
        Some(Paginator::Token(TokenPair {
            input: MemberPath::selected("NextToken")?,
            output: MemberPath::selected("NextToken")?,
            optional: false,
        }))
    );
    Ok(())
}

#[test]
fn global_accelerators_are_account_scoped_and_private_components_are_owner_filtered() -> TestResult
{
    let policies = crate::overlay::embedded()?;
    let accelerators = operation(&policies, "global-accelerator", "ListAccelerators")?;
    let components = operation(&policies, "greengrassv2", "ListComponents")?;
    assert_eq!(accelerators.scope, Some(Scope::Account));
    assert_eq!(
        components.fixed.get(&MemberPath::selected("scope")?),
        Some(&FixedValue::String("PRIVATE".to_owned()))
    );
    Ok(())
}

#[test]
fn legacy_analytics_cursor_blocker_is_explicit_not_falsely_supported() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let decision = operation(&policies, "kinesis-analytics", "ListApplications")?;
    assert_eq!(decision.class, Some(OverlayClass::Defer));
    assert_eq!(decision.items, None);
    assert!(
        decision
            .defer_reason
            .as_deref()
            .unwrap_or_default()
            .contains("ExclusiveStartApplicationName")
    );
    Ok(())
}

#[test]
fn greengrass_deployment_history_is_explicitly_complete() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let deployments = operation(&policies, "greengrassv2", "ListDeployments")?;
    assert_eq!(
        deployments.fixed.get(&MemberPath::new("historyFilter")?),
        Some(&FixedValue::String("ALL".to_owned()))
    );
    Ok(())
}

#[test]
fn safe_profile_and_workspace_entity_chains_are_callable_without_content_reads() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let profiles = operation(&policies, "iot", "ListSecurityProfiles")?;
    assert_eq!(profiles.class, Some(OverlayClass::Enumerate));
    assert_eq!(
        profiles.identity,
        Some(Identity::Arn(MemberPath::new("arn")?))
    );
    let entities = operation(&policies, "iottwinmaker", "ListEntities")?;
    assert_eq!(entities.class, Some(OverlayClass::Expand));
    assert_eq!(
        entities.bind.get(&MemberPath::new("workspaceId")?),
        Some(&Binding::From {
            operation: OperationName::new("ListWorkspaces")?,
            path: MemberPath::new("workspaceId")?
        })
    );
    assert_eq!(
        entities.paginator,
        Some(Paginator::Token(TokenPair {
            input: MemberPath::new("nextToken")?,
            output: MemberPath::new("nextToken")?,
            optional: false,
        }))
    );
    for operation_name in [
        "ExecuteQuery",
        "GetPropertyValue",
        "GetPropertyValueHistory",
    ] {
        assert_eq!(
            operation(&policies, "iottwinmaker", operation_name)?.class,
            Some(OverlayClass::Deny(DenyReason::Content))
        );
    }
    Ok(())
}

#[test]
fn iot_metric_and_dimension_names_are_complete_regional_scalar_roots_without_namespace_inputs()
-> TestResult {
    let policies = crate::overlay::embedded()?;
    for (name, items, prefix) in [
        ("ListCustomMetrics", "metricNames", "custommetric"),
        ("ListDimensions", "dimensionNames", "dimension"),
    ] {
        let plan = operation(&policies, "iot", name)?;
        assert_eq!(plan.class, Some(OverlayClass::Enumerate));
        assert_eq!(plan.scope, Some(Scope::Region));
        assert_eq!(
            plan.items,
            Some(ItemSelector::Member(MemberPath::selected(items)?))
        );
        assert_eq!(plan.queryable, vec![MemberPath::selected("$")?]);
        assert_eq!(
            plan.identity,
            Some(Identity::Template(format!(
                "arn:{{partition}}:iot:{{region}}:{{account}}:{prefix}/{{$}}"
            )))
        );
        assert_eq!(
            plan.paginator,
            Some(Paginator::Token(TokenPair {
                input: MemberPath::new("nextToken")?,
                output: MemberPath::new("nextToken")?,
                optional: false,
            }))
        );
        assert!(plan.bind.is_empty());
        assert!(plan.fixed.is_empty());
    }
    for name in ["DescribeCustomMetric", "DescribeDimension"] {
        assert_eq!(
            operation(&policies, "iot", name)?.class,
            Some(OverlayClass::Defer)
        );
    }
    Ok(())
}

#[test]
fn keyspace_type_names_keep_selected_roots_and_keyspace_request_correlation() -> TestResult {
    let policies = crate::overlay::embedded()?;
    let plan = operation(&policies, "keyspaces", "ListTypes")?;
    assert_eq!(
        plan.identity,
        Some(Identity::CorrelatedKey(vec![
            crate::overlay::IdentityField::Request(MemberPath::new("keyspaceName")?),
            crate::overlay::IdentityField::Selected(MemberPath::selected("$")?),
        ]))
    );
    assert_eq!(plan.queryable, vec![MemberPath::selected("$")?]);
    assert_eq!(
        plan.bind.get(&MemberPath::new("keyspaceName")?),
        Some(&Binding::From {
            operation: OperationName::new("ListKeyspaces")?,
            path: MemberPath::selected("keyspaceName")?,
        })
    );
    assert_eq!(plan.class, Some(OverlayClass::Expand));
    Ok(())
}

#[test]
fn scalar_selected_paths_do_not_make_request_members_or_tokens_valid() -> TestResult {
    let plan = super::policy::enumerate("names", super::policy::key("$")?, None)?;
    let error = crate::overlay::NameError {
        value: "$".to_owned(),
    };
    assert_eq!(
        super::policy::bind(plan.clone(), "$", "ListNames", "$"),
        Err(error.clone())
    );
    assert_eq!(super::policy::fixed(plan, "$", "value"), Err(error.clone()));
    assert_eq!(
        super::policy::enumerate("names", super::policy::key("$")?, Some(("$", "nextToken"))),
        Err(error.clone())
    );
    assert_eq!(
        super::policy::enumerate("names", super::policy::key("$")?, Some(("nextToken", "$"))),
        Err(error)
    );
    Ok(())
}

#[test]
fn full_builtin_assembly_keeps_guardduty_scalar_filters_and_unrelated_families_callable()
-> TestResult {
    let policies = crate::overlay::embedded()?;
    let filters = operation(&policies, "guardduty", "ListFilters")?;
    assert_eq!(
        filters.identity,
        Some(Identity::CorrelatedKey(vec![
            crate::overlay::IdentityField::Request(MemberPath::new("DetectorId")?),
            crate::overlay::IdentityField::Selected(MemberPath::selected("$")?),
        ]))
    );
    assert_eq!(
        filters.bind.get(&MemberPath::new("DetectorId")?),
        Some(&Binding::From {
            operation: OperationName::new("ListDetectors")?,
            path: MemberPath::selected("$")?,
        })
    );
    assert_eq!(filters.queryable, vec![MemberPath::selected("$")?]);
    assert_eq!(filters.class, Some(OverlayClass::Expand));
    assert_eq!(
        operation(&policies, "databrew", "ListJobs")?.class,
        Some(OverlayClass::Enumerate)
    );
    assert_eq!(
        operation(&policies, "guardduty", "GetFindings")?.class,
        Some(OverlayClass::Deny(DenyReason::Content))
    );
    Ok(())
}
