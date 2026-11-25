use catalog_tool::{
    classes::DenyList, endpoints, kind_map, names, overlay, runtime_catalog, smithy,
};
use inventory_core::catalog::runtime::RuntimeCatalog;
use std::{collections::BTreeMap, fs, path::Path};
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[derive(Clone, Copy)]
pub enum Case {
    Ordinary,
    Skipped,
    AdditionalEmission,
    CrossService,
}
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Skipruntime":{"type":"service","version":"1","operations":[{"target":"n#GetCallerIdentity"},{"target":"n#DescribeRegions"},{"target":"n#ListThings"},{"target":"n#DescribeThing"}],"traits":{"aws.api#service":{"sdkId":"Skipruntime","endpointPrefix":"skipruntime"},"aws.auth#sigv4":{"name":"skipruntime"},"aws.protocols#awsJson1_1":{}}},
"n#GetCallerIdentity":{"type":"operation","output":{"target":"n#AccountOutput"}},
"n#AccountOutput":{"type":"structure","members":{"Account":{"target":"smithy.api#String"}}},
"n#DescribeRegions":{"type":"operation","output":{"target":"n#RegionsOutput"}},
"n#RegionsOutput":{"type":"structure","members":{"Regions":{"target":"n#Regions"}}},
"n#Regions":{"type":"list","member":{"target":"n#Region"}},
"n#Region":{"type":"structure","members":{"RegionName":{"target":"smithy.api#String"}}},
"n#ListThings":{"type":"operation","output":{"target":"n#Output"}},
"n#DescribeThing":{"type":"operation","input":{"target":"n#Input"},"output":{"target":"n#Output"}},
"n#Input":{"type":"structure","members":{"Id":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}},
"n#Output":{"type":"structure","members":{"Account":{"target":"smithy.api#String"},"Items":{"target":"n#Items"}}},
"n#Items":{"type":"list","member":{"target":"n#Item"}},
"n#Item":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Count":{"target":"smithy.api#Long"},"Tags":{"target":"n#Tags"}}},
"n#Tags":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}}
}}"#;

fn plan(
    scope: overlay::Scope,
    items: &str,
    identity: &str,
) -> TestResult<overlay::OperationOverlay> {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(scope);
    plan.items = Some(overlay::ItemSelector::parse(items)?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        identity,
    )?]));
    plan.paginator = Some(overlay::Paginator::None);
    Ok(plan)
}

fn policy(
    case: Case,
    evidence: Option<&overlay::SkipEvidence>,
    customize: &impl Fn(&mut overlay::OperationOverlay),
) -> TestResult<overlay::ServiceOverlay> {
    let mut parent = plan(overlay::Scope::Region, "Items", "Id")?;
    if let Some(evidence) = evidence {
        let child = overlay::OperationName::new("DescribeThing")?;
        parent.skip.push(child.clone());
        parent.skip_evidence.insert(child, evidence.clone());
    }
    if matches!(case, Case::AdditionalEmission) {
        parent
            .additional_resources
            .push(overlay::ResourceEmission::new(
                "child",
                overlay::ItemSelector::parse("Items")?,
                overlay::Identity::Key(vec![overlay::MemberPath::new("Id")?]),
            ));
    }
    let mut child = plan(overlay::Scope::Region, "Items", "Id")?;
    child.class = Some(overlay::OverlayClass::Expand);
    child.bind.insert(
        overlay::MemberPath::new("Id")?,
        overlay::Binding::From {
            operation: overlay::OperationName::new("ListThings")?,
            path: overlay::MemberPath::new("Id")?,
        },
    );
    customize(&mut child);
    Ok(overlay::ServiceOverlay::new(
        "skipruntime",
        vec![
            (
                "GetCallerIdentity",
                plan(overlay::Scope::Account, "$", "Account")?,
            ),
            (
                "DescribeRegions",
                plan(overlay::Scope::Region, "Regions", "RegionName")?,
            ),
            ("ListThings", parent),
            ("DescribeThing", child),
        ],
    )?)
}

const API_KINDS: &[kind_map::ApiOnlyMapping<'static>] = &[
    kind_map::ApiOnlyMapping {
        kind: "AWS::ApiOnly::SkipAccount",
        reason: "Bootstrap",
    },
    kind_map::ApiOnlyMapping {
        kind: "AWS::ApiOnly::SkipRegion",
        reason: "Bootstrap",
    },
    kind_map::ApiOnlyMapping {
        kind: "AWS::ApiOnly::SkipThing",
        reason: "Skip contract",
    },
    kind_map::ApiOnlyMapping {
        kind: "AWS::ApiOnly::SkipChild",
        reason: "Additional emission contract",
    },
];
const BOOTSTRAPS: &[kind_map::BootstrapMapping<'static>] = &[
    kind_map::BootstrapMapping::AccountIdentity {
        service: "skipruntime",
        operation: "GetCallerIdentity",
        account_path: "Account",
    },
    kind_map::BootstrapMapping::RegionDiscovery {
        service: "skipruntime",
        operation: "DescribeRegions",
        regions_path: "Regions",
        region_name_path: "RegionName",
        opt_in_status_path: None,
    },
];

fn kinds(
    case: Case,
    bootstrap: Option<kind_map::BootstrapMapping<'static>>,
) -> TestResult<kind_map::KindMap> {
    let additional = [kind_map::AdditionalEmissionMapping {
        service: "skipruntime",
        operation: "ListThings",
        emitter_key: "child",
        kind: "AWS::ApiOnly::SkipChild",
    }];
    let mut bootstraps = BOOTSTRAPS.to_vec();
    if let Some(bootstrap) = bootstrap {
        let purpose = std::mem::discriminant(&bootstrap);
        bootstraps.retain(|entry| std::mem::discriminant(entry) != purpose);
        bootstraps.push(bootstrap);
    }
    Ok(kind_map::MappingSource {
        api_only: API_KINDS,
        operations: &[
            kind_map::OperationMapping {
                service: "skipruntime",
                operation: "GetCallerIdentity",
                kind: "AWS::ApiOnly::SkipAccount",
            },
            kind_map::OperationMapping {
                service: "skipruntime",
                operation: "DescribeRegions",
                kind: "AWS::ApiOnly::SkipRegion",
            },
            kind_map::OperationMapping {
                service: "skipruntime",
                operation: "ListThings",
                kind: "AWS::ApiOnly::SkipThing",
            },
            kind_map::OperationMapping {
                service: "skipruntime",
                operation: "DescribeThing",
                kind: if matches!(case, Case::AdditionalEmission) {
                    "AWS::ApiOnly::SkipChild"
                } else {
                    "AWS::ApiOnly::SkipThing"
                },
            },
            kind_map::OperationMapping {
                service: "otherskip",
                operation: "DescribeThing",
                kind: "AWS::ApiOnly::SkipChild",
            },
        ],
        bootstraps: &bootstraps,
        additional_emissions: if matches!(case, Case::AdditionalEmission) {
            &additional
        } else {
            &[]
        },
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?)
}

pub fn catalog(case: Case) -> TestResult<Vec<u8>> {
    let ordinary = catalog_with(case, None, None, |_| {})?;
    if matches!(case, Case::Ordinary) {
        return Ok(ordinary);
    }
    let ordinary = RuntimeCatalog::from_bytes(&ordinary)?;
    let evidence = receipt(&ordinary)?;
    catalog_with(case, Some(&evidence), None, |_| {})
}

pub fn receipt(catalog: &RuntimeCatalog<'_>) -> TestResult<overlay::SkipEvidence> {
    let fingerprint =
        overlay::SkipEvidence::fingerprint(catalog, "skipruntime", "ListThings", "DescribeThing")?;
    Ok(overlay::SkipEvidence::new(
        "skipruntime",
        "ListThings",
        "DescribeThing",
        &fingerprint,
        "seeded members and raw values",
        1,
        "2026-11-24",
    )?)
}

pub fn catalog_with(
    case: Case,
    evidence: Option<&overlay::SkipEvidence>,
    bootstrap: Option<kind_map::BootstrapMapping<'static>>,
    customize: impl Fn(&mut overlay::OperationOverlay),
) -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let models = root.path().join("models/skipruntime");
    fs::create_dir_all(&models)?;
    fs::write(models.join("model.json"), MODEL)?;
    let mut policies = vec![policy(case, evidence, &customize)?];
    let mut endpoints = BTreeMap::from([("skipruntime".to_owned(), endpoint("skipruntime")?)]);
    if matches!(case, Case::CrossService) {
        let other = root.path().join("models/otherskip");
        fs::create_dir_all(&other)?;
        fs::write(
            other.join("model.json"),
            MODEL
                .replace("n#", "o#")
                .replace("Skipruntime", "Otherskip")
                .replace("skipruntime", "otherskip"),
        )?;
        policies.push(cross_service_policy()?);
        endpoints.insert("otherskip".into(), endpoint("otherskip")?);
    }
    let checked = overlay::check_policies(root.path(), &policies, &DenyList::builtin())?;
    let fields = smithy::Registry::parse("Account\nCount\nId\nItems\nRegionName\nRegions\nTags\n")?;
    let base = smithy::build_catalog(root.path(), &fields, "skip-runtime-contract")?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds(case, bootstrap)?,
        &endpoints,
    )?)
}

fn endpoint(service: &str) -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = serde_json::json!({"authSchemes":[{"name":"sigv4","signingName":service,"signingRegion":"us-east-1"}]});
    let url = format!("https://{service}.invalid");

    let document = serde_json::json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":url,"properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("skip-endpoint.json"), &document)?;
    let cases = serde_json::from_value::<Vec<endpoints::Case>>(serde_json::json!([
        {"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":url,"properties":properties}}}
    ]))?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some(service))
        .map_err(|error| format!("fixture endpoint error: {error:?}").into())
}
fn cross_service_policy() -> TestResult<overlay::ServiceOverlay> {
    let mut child = plan(overlay::Scope::Region, "Items", "Id")?;
    child.class = Some(overlay::OverlayClass::Expand);
    child.bind.insert(
        overlay::MemberPath::new("Id")?,
        overlay::Binding::FromService {
            service: "skipruntime".into(),
            operation: overlay::OperationName::new("ListThings")?,
            path: overlay::MemberPath::new("Id")?,
        },
    );
    let mut entries = vec![("DescribeThing", child)];
    entries.extend(
        ["GetCallerIdentity", "DescribeRegions", "ListThings"].map(|name| {
            let mut plan = overlay::OperationOverlay::default();
            plan.class = Some(overlay::OverlayClass::Defer);
            plan.defer_reason =
                Some("Cross-service fixture consumes the primary service only".into());
            (name, plan)
        }),
    );
    Ok(overlay::ServiceOverlay::new("otherskip", entries)?)
}
