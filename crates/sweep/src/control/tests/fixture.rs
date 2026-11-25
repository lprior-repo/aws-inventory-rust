use super::*;
use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use std::{collections::BTreeMap, path::Path};

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#GetCallerIdentity"},{"target":"n#DescribeRegions"},{"target":"n#ListItems"}],"traits":{"aws.api#service":{"sdkId":"Controlinventory","endpointPrefix":"controlinventory","arnNamespace":"controlinventory"},"aws.auth#sigv4":{"name":"controlinventory"},"aws.protocols#restJson1":{}}},
"n#GetCallerIdentity":{"type":"operation","output":{"target":"n#IdentityOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/identity"}}},
"n#IdentityOutput":{"type":"structure","members":{"Account":{"target":"smithy.api#String"}}},
"n#DescribeRegions":{"type":"operation","output":{"target":"n#RegionsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/regions"}}},
"n#RegionsOutput":{"type":"structure","members":{"Regions":{"target":"n#RegionsList"}}},
"n#RegionsList":{"type":"list","member":{"target":"n#Region"}},
"n#Region":{"type":"structure","members":{"RegionName":{"target":"smithy.api#String"}}},
"n#ListItems":{"type":"operation","input":{"target":"n#ItemsInput"},"output":{"target":"n#ItemsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/items"}}},
"n#ItemsInput":{"type":"structure","members":{"Marker":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"marker"}}}},
"n#ItemsOutput":{"type":"structure","members":{"Items":{"target":"n#Items"},"NextToken":{"target":"smithy.api#String"}}},
"n#Items":{"type":"list","member":{"target":"n#Item"}},
"n#UnusedInput":{"type":"structure","members":{"Arn":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#httpQuery":"arn"}}}},
"n#Item":{"type":"structure","members":{"Arn":{"target":"smithy.api#String"},"SubscriptionArn":{"target":"smithy.api#String"}}}
}}"#;

const KINDS: kind_map::MappingSource<'static> = kind_map::MappingSource {
    version: 1,
    reviewed: "2026-11-23",
    reviewer: "Cancellation and progress contract",
    default_unmapped_reason: None,
    cfn: &[],
    relations: &[],
    parent_relations: &[],
    compound_relations: &[],
    additional_emissions: &[],
    relation_targets: &[],
    api_only: &[
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::ControlAccount",
            reason: "Bootstrap account",
        },
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::ControlRegion",
            reason: "Bootstrap region",
        },
        kind_map::ApiOnlyMapping {
            kind: "AWS::ApiOnly::ControlItem",
            reason: "Pagination contract",
        },
    ],
    operations: &[
        kind_map::OperationMapping {
            service: "controlinventory",
            operation: "GetCallerIdentity",
            kind: "AWS::ApiOnly::ControlAccount",
        },
        kind_map::OperationMapping {
            service: "controlinventory",
            operation: "DescribeRegions",
            kind: "AWS::ApiOnly::ControlRegion",
        },
        kind_map::OperationMapping {
            service: "controlinventory",
            operation: "ListItems",
            kind: "AWS::ApiOnly::ControlItem",
        },
    ],
    bootstraps: &[
        kind_map::BootstrapMapping::AccountIdentity {
            service: "controlinventory",
            operation: "GetCallerIdentity",
            account_path: "Account",
        },
        kind_map::BootstrapMapping::RegionDiscovery {
            service: "controlinventory",
            operation: "DescribeRegions",
            regions_path: "Regions",
            region_name_path: "RegionName",
            opt_in_status_path: None,
        },
    ],
};

fn operation(
    scope: overlay::Scope,
    items: &str,
    identity: &str,
) -> TestResult<overlay::OperationOverlay> {
    let mut operation = overlay::OperationOverlay::default();
    operation.class = Some(overlay::OverlayClass::Enumerate);
    operation.scope = Some(scope);
    operation.items = Some(overlay::ItemSelector::parse(items)?);
    operation.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::selected(
        identity,
    )?]));
    operation.paginator = Some(overlay::Paginator::None);
    Ok(operation)
}

fn policy(unused: bool) -> TestResult<overlay::ServiceOverlay> {
    let mut items = operation(overlay::Scope::Region, "Items", "Arn")?;
    items.paginator = Some(overlay::Paginator::Tokens {
        pairs: vec![overlay::TokenPair::new(
            overlay::MemberPath::new("Marker")?,
            overlay::MemberPath::new("NextToken")?,
        )],
        truncated: None,
    });
    let mut operations = vec![
        (
            "GetCallerIdentity",
            operation(overlay::Scope::Account, "$", "Account")?,
        ),
        (
            "DescribeRegions",
            operation(overlay::Scope::Region, "Regions", "RegionName")?,
        ),
        ("ListItems", items),
    ];
    if unused {
        let mut expansion = operation(overlay::Scope::Region, "Items", "Arn")?;
        expansion.class = Some(overlay::OverlayClass::Expand);
        expansion.bind.insert(
            overlay::MemberPath::new("Arn")?,
            overlay::Binding::FromArn {
                operation: overlay::OperationName::new("ListItems")?,
                path: overlay::MemberPath::new("SubscriptionArn")?,
                arn_service: "sns".to_owned(),
                resource_kind: overlay::ArnResourceKind::SnsSubscription,
            },
        );
        operations.push(("ListUnused", expansion));
    }
    Ok(overlay::ServiceOverlay::new(
        "controlinventory",
        operations,
    )?)
}

pub(super) fn catalog(root: &Path, unused: bool) -> TestResult<Vec<u8>> {
    let source = root.join("source");
    let models = source.join("models/controlinventory/service/2026-01-01");
    std::fs::create_dir_all(&models)?;
    let mut model: serde_json::Value = serde_json::from_str(MODEL)?;
    if unused {
        model
            .pointer_mut("/shapes/n#Service/operations")
            .and_then(serde_json::Value::as_array_mut)
            .ok_or("fixture service operations absent")?
            .push(serde_json::json!({"target":"n#ListUnused"}));
        model
            .pointer_mut("/shapes")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or("fixture shapes absent")?
            .insert(
                "n#ListUnused".into(),
                serde_json::json!({"type":"operation","input":{"target":"n#UnusedInput"},
                    "output":{"target":"n#ItemsOutput"},
                    "traits":{"smithy.api#http":{"method":"GET","uri":"/unused"}}}),
            );
    }
    std::fs::write(models.join("model.json"), serde_json::to_vec(&model)?)?;
    let checked = overlay::check_policies(
        &source,
        &[policy(unused)?],
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let registry = smithy::Registry::parse(
        "Account\nArn\nItems\nMarker\nNextToken\nRegionName\nRegions\nSubscriptionArn\n",
    )?;
    let base = smithy::build_catalog(&source, &registry, "control-contract")?;
    let mut operation_kinds = KINDS.operations.to_vec();
    if unused {
        operation_kinds.push(kind_map::OperationMapping {
            service: "controlinventory",
            operation: "ListUnused",
            kind: "AWS::ApiOnly::ControlItem",
        });
    }
    let kinds = kind_map::MappingSource {
        operations: &operation_kinds,
        ..KINDS
    }
    .compile(&names::Exceptions::none())?;
    let endpoints = BTreeMap::from([("controlinventory".to_owned(), endpoint()?)]);
    Ok(runtime_catalog::augment(
        &base, &checked, &kinds, &endpoints,
    )?)
}

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = serde_json::json!({"authSchemes":[{"name":"sigv4","signingName":"controlinventory","signingRegion":"us-east-1"}]});
    let document = serde_json::json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://controlinventory.invalid","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("control-endpoint.json"), &document)?;
    let cases = serde_json::from_value::<Vec<endpoints::Case>>(serde_json::json!([
        {"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://controlinventory.invalid","properties":properties}}}
    ]))?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("controlinventory"))
        .map_err(|error| format!("fixture endpoint error: {error:?}").into())
}
