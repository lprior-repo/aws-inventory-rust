use catalog_tool::{
    classes::DenyList, endpoints, kind_map, names, overlay, runtime_catalog, smithy,
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
pub const SOURCE: &str = "AWS::ApiOnly::ReferenceSource";
pub const TARGET: &str = "AWS::ApiOnly::ReferenceTarget";
pub const OTHER_SOURCE: &str = "AWS::ApiOnly::OtherReferenceSource";
pub const THIN_SERVICE: &str = "ec2";

pub enum TargetState {
    Callable,
    Deferred,
    Compound,
    Collections,
    MissingMember,
    RepeatedIdentity,
    Thin,
    DistinctBindings,
    InactiveReview,
    SelfReference,
    IdentityAndSeparateReference,
    Branching,
    ArnKey,
    ArnString,
    DeepReference,
    BoundaryReference,
    TwoSources,
    DeepStructureReference,
    TruncatedStructureReference,
    CyclicReference,
    FanoutReference,
    SmithyStringKey,
    InvalidStringIds,
    CollidingPending,
}

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","traits":{"aws.api#service":{"sdkId":"Refs","endpointPrefix":"refs"},"aws.auth#sigv4":{"name":"refs"},"aws.protocols#awsJson1_1":{}}},
"n#ListSources":{"type":"operation","output":{"target":"n#SourceOutput"}},
"n#ListTargets":{"type":"operation","output":{"target":"n#TargetOutput"}},
"n#ListPending":{"type":"operation","output":{"target":"n#TargetOutput"}},
"n#SourceOutput":{"type":"structure","members":{"SourceItems":{"target":"n#Sources"}}},
"n#TargetOutput":{"type":"structure","members":{"TargetItems":{"target":"n#Targets"}}},
"n#Sources":{"type":"list","member":{"target":"n#Source"}},
"n#Targets":{"type":"list","member":{"target":"n#Target"}},
"n#Source":{"type":"structure","traits":{"smithy.api#references":[{"resource":"n#Resource","ids":{"Id":"IdRef"}}]},"members":{
"Id":{"target":"smithy.api#String"},
"IdRef":{"target":"smithy.api#String"},
"ArnRef":{"target":"smithy.api#String","traits":{"aws.api#arnReference":{"resource":"n#Resource"}}},
"Nested":{"target":"n#Nested"},
"PendingRef":{"target":"smithy.api#String","traits":{"aws.api#arnReference":{"resource":"n#Pending"}}},
"ExternalRef":{"target":"smithy.api#String","traits":{"aws.api#arnReference":{"resource":"external#Resource"}}}}},
"n#Nested":{"type":"structure","traits":{"smithy.api#references":[{"resource":"n#Resource"}]},"members":{"Id":{"target":"smithy.api#String"}}},
"n#Target":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}},
"n#Resource":{"type":"resource","identifiers":{"Id":{"target":"smithy.api#String"}},"list":{"target":"n#ListTargets"},"read":{"target":"n#ListTargets"}},
"n#Pending":{"type":"resource","identifiers":{"Id":{"target":"smithy.api#String"}},"list":{"target":"n#ListPending"}}
}}"#;

pub fn compile(
    target: TargetState,
    relations: &[kind_map::RelationMapping<'_>],
) -> TestResult<(Vec<u8>, runtime_catalog::encode::RelationsResult)> {
    let root = tempfile::tempdir()?;
    let service = if matches!(target, TargetState::Thin) {
        THIN_SERVICE
    } else {
        "refs"
    };
    let directory = root
        .path()
        .join(format!("models/{service}/service/2020-01-01"));
    fs::create_dir_all(&directory)?;
    let model = fixture_model(&target)?.replace("\"refs\"", &format!("{service:?}"));
    fs::write(directory.join("model.json"), model.as_bytes())?;
    let mut target_plan = arn_plan("TargetItems")?;
    if matches!(target, TargetState::Deferred) {
        target_plan = deferred();
    }
    if matches!(
        target,
        TargetState::Compound | TargetState::DistinctBindings
    ) {
        target_plan.identity = Some(overlay::Identity::Key(vec![
            overlay::MemberPath::new("Id")?,
            overlay::MemberPath::new("IdRef")?,
        ]));
    }
    if matches!(target, TargetState::RepeatedIdentity) {
        target_plan.identity = Some(overlay::Identity::Key(vec![
            overlay::MemberPath::new("Nested.Id")?,
            overlay::MemberPath::new("Other.Id")?,
        ]));
    }
    let mut operations = vec![
        ("ListSources", source_plan(&target)?),
        ("ListTargets", target_plan),
        ("ListPending", deferred()),
    ];
    extra_operations(&target, &mut operations)?;
    let policies = [overlay::ServiceOverlay::new(service, operations)?];
    let checked = overlay::check_policies(root.path(), &policies, &DenyList::builtin())?;
    let registry = smithy::Registry::parse(
        "ArnRef\nExternalRef\nId\nIdRef\nNested\nPendingRef\nSourceItems\nTargetItems\nCollection\nMapped\nOther\n",
    )?;
    let base = smithy::build_catalog(root.path(), &registry, "declared-fixture")?;
    let kinds = kinds(relations, service, &target)?;
    let endpoints = BTreeMap::from([(service.to_owned(), endpoint()?)]);
    Ok(runtime_catalog::augment_with_sources(
        &base,
        &checked,
        &kinds,
        &endpoints,
        &runtime_catalog::CatalogSources::compiled(&base),
    )?)
}

fn kinds(
    relations: &[kind_map::RelationMapping<'_>],
    service: &str,
    target: &TargetState,
) -> TestResult<kind_map::KindMap> {
    let mut operations = vec![
        kind_map::OperationMapping {
            service,
            operation: "ListSources",
            kind: SOURCE,
        },
        kind_map::OperationMapping {
            service,
            operation: "ListTargets",
            kind: TARGET,
        },
    ];
    extra_kinds(target, service, &mut operations);
    Ok(kind_map::MappingSource {
        api_only: &[
            kind_map::ApiOnlyMapping {
                kind: SOURCE,
                reason: "Reference source fixture",
            },
            kind_map::ApiOnlyMapping {
                kind: TARGET,
                reason: "Reference target fixture",
            },
            kind_map::ApiOnlyMapping {
                kind: OTHER_SOURCE,
                reason: "Second reference source fixture",
            },
            kind_map::ApiOnlyMapping {
                kind: "AWS::ApiOnly::MissingReferenceTarget",
                reason: "Inactive target fixture",
            },
        ],
        operations: &operations,
        relations,
        relation_targets: if matches!(target, TargetState::InactiveReview) {
            &[kind_map::RelationTargetsMapping {
                source_kind: SOURCE,
                source: kind_map::RelationSource::SelectedOutput(kind_map::RelationPath::Fields(
                    "ArnRef",
                )),
                relation: "REVIEWED_REFERENCE",
                target_kinds: &[TARGET, "AWS::ApiOnly::MissingReferenceTarget"],
            }]
        } else {
            &[]
        },
        ..kind_map::MappingSource::default()
    }
    .compile(&names::Exceptions::none())?)
}

fn plan(items: &str) -> TestResult<overlay::OperationOverlay> {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Enumerate);
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse(items)?);
    plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
        "Id",
    )?]));
    plan.paginator = Some(overlay::Paginator::None);
    Ok(plan)
}

fn deferred() -> overlay::OperationOverlay {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(overlay::OverlayClass::Defer);
    plan.defer_reason = Some("Reference target fixture is not callable".to_owned());
    plan
}

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties =
        json!({"authSchemes":[{"name":"sigv4","signingName":"refs","signingRegion":"us-east-1"}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://refs.test","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("reference-endpoint.json"), &document)?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(
        json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://refs.test","properties":properties}}}]),
    )?;
    rules
        .run_cases(&cases)?
        .verified_endpoint(Some("refs"))
        .map_err(|failure| format!("fixture endpoint evidence failed: {failure:?}").into())
}

fn arn_plan(items: &str) -> TestResult<overlay::OperationOverlay> {
    let mut result = plan(items)?;
    result.identity = Some(overlay::Identity::Arn(overlay::MemberPath::new("Id")?));
    Ok(result)
}

fn source_plan(target: &TargetState) -> TestResult<overlay::OperationOverlay> {
    if matches!(
        target,
        TargetState::SelfReference | TargetState::IdentityAndSeparateReference
    ) {
        arn_plan("SourceItems")
    } else {
        plan("SourceItems")
    }
}

fn extra_operations(
    target: &TargetState,
    operations: &mut Vec<(&'static str, overlay::OperationOverlay)>,
) -> TestResult {
    if matches!(
        target,
        TargetState::ArnKey | TargetState::SmithyStringKey | TargetState::CollidingPending
    ) {
        let (_, plan) = operations
            .iter_mut()
            .find(|(name, _)| *name == "ListTargets")
            .ok_or("target operation missing")?;
        plan.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::new(
            "Id",
        )?]));
    }
    if matches!(target, TargetState::TwoSources) {
        operations.push(("ListOthers", plan("SourceItems")?));
    }
    Ok(())
}

fn extra_kinds<'a>(
    target: &TargetState,
    service: &'a str,
    operations: &mut Vec<kind_map::OperationMapping<'a>>,
) {
    if matches!(target, TargetState::TwoSources) {
        operations.push(kind_map::OperationMapping {
            service,
            operation: "ListOthers",
            kind: OTHER_SOURCE,
        });
    }
}

fn compound_model() -> TestResult<String> {
    let mut document: serde_json::Value = serde_json::from_str(MODEL)?;
    *document
        .pointer_mut("/shapes/n#Source/traits/smithy.api#references/0/ids")
        .ok_or("source reference identifiers missing")? = json!({"Id":"IdRef", "IdRef":"Id"});
    [
        "/shapes/n#Target/members",
        "/shapes/n#Nested/members",
        "/shapes/n#Resource/identifiers",
    ]
    .into_iter()
    .try_for_each(|pointer| -> TestResult {
        document
            .pointer_mut(pointer)
            .and_then(serde_json::Value::as_object_mut)
            .ok_or("fixture identifier members missing")?
            .insert("IdRef".to_owned(), json!({"target":"smithy.api#String"}));
        Ok(())
    })?;
    Ok(serde_json::to_string(&document)?)
}

fn fixture_model(target: &TargetState) -> TestResult<String> {
    let base = if matches!(
        target,
        TargetState::Compound | TargetState::DistinctBindings
    ) {
        compound_model()?
    } else {
        MODEL.to_owned()
    };
    let mut document: serde_json::Value = serde_json::from_str(&base)?;
    match target {
        TargetState::SelfReference | TargetState::IdentityAndSeparateReference => {
            object(&mut document, "/shapes/n#Resource")?.extend([
                ("list".into(), json!({"target":"n#ListSources"})),
                ("read".into(), json!({"target":"n#ListSources"})),
            ]);
            if matches!(target, TargetState::IdentityAndSeparateReference) {
                object(&mut document, "/shapes/n#Source/members/Id")?.insert(
                    "traits".into(),
                    json!({"aws.api#arnReference":{"resource":"n#Resource"}}),
                );
            }
        }
        TargetState::Collections => {
            object(&mut document, "/shapes/n#Source/members")?
                .insert("Collection".into(), json!({"target":"n#NestedList"}));
            object(&mut document, "/shapes/n#Source/members")?
                .insert("Mapped".into(), json!({"target":"n#NestedMap"}));
            object(&mut document, "/shapes")?.extend([
                ("n#NestedList".into(), json!({"type":"list","member":{"target":"n#Nested"}})),
                ("n#NestedMap".into(), json!({"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"n#Nested"}})),
            ]);
        }
        TargetState::MissingMember => {
            object(&mut document, "/shapes/n#Nested/members")?.remove("Id");
        }
        TargetState::RepeatedIdentity => {
            object(&mut document, "/shapes/n#Target/members")?.extend([
                ("Nested".into(), json!({"target":"n#IdentityPart"})),
                ("Other".into(), json!({"target":"n#IdentityPart"})),
            ]);
            object(&mut document, "/shapes")?.insert(
                "n#IdentityPart".into(),
                json!({"type":"structure","members":{"Id":{"target":"smithy.api#String"}}}),
            );
        }
        TargetState::Thin => {
            object(&mut document, "/shapes/n#Source/traits")?.remove("smithy.api#references");
            object(&mut document, "/shapes/n#Nested/traits")?.remove("smithy.api#references");
        }
        TargetState::DistinctBindings => {
            *document
                .pointer_mut("/shapes/n#Source/traits/smithy.api#references")
                .ok_or("source reference declarations absent")? = json!([
                {"resource":"n#Resource","ids":{"Id":"IdRef","IdRef":"Id"}},
                {"resource":"n#Resource","ids":{"Id":"IdRef","IdRef":"ArnRef"}},
            ]);
        }
        _ => {}
    }
    extra_model(target, &mut document)?;
    Ok(serde_json::to_string(&document)?)
}

fn extra_model(target: &TargetState, document: &mut serde_json::Value) -> TestResult {
    if matches!(target, TargetState::Branching) {
        object(document, "/shapes/n#Source/members")?
            .insert("Other".into(), json!({"target":"n#Branch0"}));
        for index in 0_usize..17 {
            let next = if index == 16 {
                "smithy.api#String".to_owned()
            } else {
                format!("n#Branch{}", index.saturating_add(1))
            };
            object(document, "/shapes")?.insert(
                format!("n#Branch{index}"),
                json!({"type":"structure","members":{"Id":{"target":next},"Nested":{"target":next}}}),
            );
        }
    }
    if matches!(
        target,
        TargetState::ArnKey
            | TargetState::ArnString
            | TargetState::SmithyStringKey
            | TargetState::InvalidStringIds
            | TargetState::CollidingPending
    ) {
        let member = object(document, "/shapes/n#Source/members/ArnRef")?;
        member.remove("traits");
        member.insert("target".into(), json!("n#ReferenceArn"));
        object(document, "/shapes")?.insert(
            "n#ReferenceArn".into(),
            json!({"type":"string","traits":{"aws.api#arnReference":{"resource":"n#Resource"}}}),
        );
    }
    if matches!(
        target,
        TargetState::DeepReference | TargetState::BoundaryReference
    ) {
        let depth = if matches!(target, TargetState::BoundaryReference) {
            7
        } else {
            8
        };
        deep_model(document, depth)?;
    }
    if matches!(target, TargetState::TwoSources) {
        object(document, "/shapes")?.insert(
            "n#ListOthers".into(),
            json!({"type":"operation","output":{"target":"n#SourceOutput"}}),
        );
    }
    regression_model(target, document)?;
    Ok(())
}

fn deep_model(document: &mut serde_json::Value, depth: usize) -> TestResult {
    object(document, "/shapes/n#Source/members")?
        .insert("Other".into(), json!({"target":"n#Deep0"}));
    for index in 0_usize..depth {
        let shape = if index.saturating_add(1) == depth {
            json!({"type":"structure","traits":{"smithy.api#references":[{"resource":"n#Resource"}]},"members":{"Id":{"target":"smithy.api#String"}}})
        } else {
            json!({"type":"structure","members":{"Nested":{"target":format!("n#Deep{}",index.saturating_add(1))}}})
        };
        object(document, "/shapes")?.insert(format!("n#Deep{index}"), shape);
    }
    Ok(())
}

fn regression_model(target: &TargetState, document: &mut serde_json::Value) -> TestResult {
    match target {
        TargetState::DeepStructureReference => deep_model(document, 9)?,
        TargetState::TruncatedStructureReference => deep_model(document, 10)?,
        TargetState::CyclicReference => {
            object(document, "/shapes/n#Source/members")?
                .insert("Other".into(), json!({"target":"n#Source"}));
        }
        TargetState::FanoutReference => fanout_model(document)?,
        TargetState::SmithyStringKey | TargetState::InvalidStringIds => {
            let reference = if matches!(target, TargetState::InvalidStringIds) {
                json!({"resource":"n#Resource","ids":{"Id":"Id"}})
            } else {
                json!({"resource":"n#Resource"})
            };
            object(document, "/shapes/n#ReferenceArn")?.insert(
                "traits".into(),
                json!({"smithy.api#references":[reference]}),
            );
        }
        TargetState::CollidingPending => {
            object(document, "/shapes/n#ReferenceArn/traits")?.insert(
                "smithy.api#references".into(),
                json!([{"resource":"n#Resource"}]),
            );
        }
        _ => {}
    }
    Ok(())
}

fn fanout_model(document: &mut serde_json::Value) -> TestResult {
    object(document, "/shapes/n#Source/members")?
        .insert("Other".into(), json!({"target":"n#Fan0"}));
    for index in 0_usize..9 {
        let next = format!("n#Fan{}", index.saturating_add(1));
        let shape = if index == 8 {
            json!({"type":"structure","traits":{"smithy.api#references":[{"resource":"n#Resource"}]},"members":{"Id":{"target":"smithy.api#String"}}})
        } else {
            json!({"type":"structure","members":{"Id":{"target":next},"IdRef":{"target":next},"Nested":{"target":next},"Other":{"target":next}}})
        };
        object(document, "/shapes")?.insert(format!("n#Fan{index}"), shape);
    }
    Ok(())
}

fn object<'a>(
    document: &'a mut serde_json::Value,
    pointer: &str,
) -> TestResult<&'a mut serde_json::Map<String, serde_json::Value>> {
    document
        .pointer_mut(pointer)
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| format!("fixture object missing at {pointer}").into())
}
