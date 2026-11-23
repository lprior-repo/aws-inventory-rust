use super::*;
use crate::record;
use catalog_tool::smithy::Registry;
use inventory_core::record::{Record, ValueRef};

const FAMILIES: &[(&str, &str, &str, &str)] = &[
    (
        "ListEnvironmentTemplates",
        "ListEnvironmentTemplateVersions",
        "environment",
        "AWS::ApiOnly::ProtonEnvironmentTemplateVersion",
    ),
    (
        "ListServiceTemplates",
        "ListServiceTemplateVersions",
        "service",
        "AWS::ApiOnly::ProtonServiceTemplateVersion",
    ),
];

#[tokio::test]
async fn given_proton_templates_when_major_discovery_and_minor_pages_resume_after_reopen_then_finite_stages_persist_every_version()
-> TestResult {
    // Given real pinned plans, two paginated templates per family, and durable work.
    let operations: Vec<_> = FAMILIES
        .iter()
        .flat_map(|family| [family.0, family.1])
        .collect();
    let bytes = compiled(
        "proton",
        "2020-07-20",
        &operations,
        "AWS::Proton::EnvironmentTemplate",
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let scenario = Scenario::new(&catalog, "proton-finite-major-minor-stages").await?;
    for family in FAMILIES {
        scenario.seed(plan(&catalog, "proton", family.0)?).await?;
    }
    // When actual template -> major discovery -> minor work resumes across reopen.
    let scenario = consume_pages(scenario, &catalog).await?.reopen().await?;
    // Then all sixteen minor resources and exact owning-template edges survive.
    // Major discovery's synthetic latest-minor 99 rows must never be emitted.
    for family in FAMILIES {
        let mut expected = BTreeSet::new();
        let mut edges = BTreeSet::new();
        for name in ["one", "two"] {
            for major in ["1", "2"] {
                for minor in ["1", "2"] {
                    let id = format!(
                        "arn:aws:proton:us-east-1:123456789012:{}-template/{name}:{major}.{minor}",
                        family.2
                    );
                    expected.insert(id.clone());
                    edges.insert((
                        id,
                        format!(
                            "arn:aws:proton:us-east-1:123456789012:{}-template/{name}",
                            family.2
                        ),
                        "VERSION_OF".to_owned(),
                    ));
                }
            }
        }
        assert_eq!(scenario.stored(family.3).await?, (expected, edges));
    }
    Ok(())
}

async fn consume_pages<'a, 'catalog>(
    mut scenario: Scenario<'a, 'catalog>,
    catalog: &'a RuntimeCatalog<'catalog>,
) -> TestResult<Scenario<'a, 'catalog>> {
    let mut seen = BTreeSet::new();
    let registry = Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    for _ in 0..28 {
        let lease = scenario.claim().await?;
        let operation = lease.scope().operation().as_str().to_owned();
        let family = FAMILIES
            .iter()
            .find(|family| family.0 == operation || family.1 == operation)
            .ok_or("unexpected Proton work")?;
        let callable = plan(catalog, "proton", &operation)?;
        let second = lease.token().is_some();
        let (request, wire) = scenario.request(callable, &lease)?;
        let native: Json = serde_json::from_slice(wire.body())?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let continuation = proton_cursor(
            &operation,
            lease.scope().branch(),
            &base,
            &native,
            &registry,
        )?;
        match native.get("nextToken") {
            Some(Json::String(token)) if second => assert_eq!(token, &continuation),
            None if !second => (),
            _ => {
                return Err(
                    "native Proton continuation does not match its independently bound scope"
                        .into(),
                );
            }
        }
        let mut body = if operation == family.0 {
            assert!(lease.scope().branch().is_none());
            let name = if second { "two" } else { "one" };
            assert!(seen.insert((operation.clone(), String::new(), String::new(), second)));
            json!({"templates":[{"name":name,"arn":format!("arn:aws:proton:us-east-1:123456789012:{}-template/{name}", family.2)}]})
        } else {
            let name = text(&native, "templateName")?;
            assert!(["one", "two"].contains(&name.as_str()));
            let stage = lease
                .scope()
                .branch()
                .ok_or("Proton version work lost finite stage")?;
            let (major, minor) = match stage {
                "majors" => {
                    assert!(
                        native.get("majorVersion").is_none(),
                        "major listing must omit its optional majorVersion filter"
                    );
                    (
                        if second {
                            "2".to_owned()
                        } else {
                            "1".to_owned()
                        },
                        "99",
                    )
                }
                "minors" => {
                    let major = text(&native, "majorVersion")?;
                    assert!(["1", "2"].contains(&major.as_str()));
                    (major, if second { "2" } else { "1" })
                }
                _ => return Err("unexpected Proton execution stage".into()),
            };
            assert!(seen.insert((
                operation.clone(),
                format!("{stage}:{name}"),
                if stage == "minors" {
                    major.clone()
                } else {
                    String::new()
                },
                second
            )));
            // A selected response templateName must not redirect the same-request binding.
            json!({"templateVersions":[{"templateName":if stage == "majors" { "not-the-requested-template" } else { name.as_str() },
                "majorVersion":major,"minorVersion":minor,"status":"PUBLISHED","createdAt":1,"lastModifiedAt":1,
                "arn":format!("arn:aws:proton:us-east-1:123456789012:{}-template/{name}:{major}.{minor}", family.2)}]})
        };
        if !second {
            body.as_object_mut()
                .ok_or("native Proton page is not an object")?
                .insert("nextToken".to_owned(), Json::String(continuation));
        }
        scenario
            .apply(catalog, callable, lease, &request, body)
            .await?;
        scenario = scenario.reopen().await?;
    }
    assert_eq!(
        seen.len(),
        28,
        "major/minor work must not recursively feed the same operation"
    );
    Ok(scenario)
}

fn proton_cursor(
    operation: &str,
    stage: Option<&str>,
    base: &Record,
    native: &Json,
    registry: &Registry,
) -> TestResult<String> {
    // Bindings are independently persisted discovery inputs, never the request token or wire.
    let (name, major) = if let Some(stage) = stage {
        let name = bound_text(base, registry, "templateName")?;
        assert_eq!(
            native.get("templateName").and_then(Json::as_str),
            Some(name)
        );
        let major = match stage {
            "majors" => {
                assert!(native.get("majorVersion").is_none());
                "all"
            }
            "minors" => {
                let major = bound_text(base, registry, "majorVersion")?;
                assert_eq!(
                    native.get("majorVersion").and_then(Json::as_str),
                    Some(major)
                );
                major
            }
            _ => return Err("unexpected Proton execution stage".into()),
        };
        (name, major)
    } else {
        assert!(native.get("templateName").is_none());
        assert!(native.get("majorVersion").is_none());
        ("all", "all")
    };
    Ok(format!(
        "resume-{operation}-{}-{name}-{major}",
        stage.unwrap_or("templates")
    ))
}

fn bound_text<'a>(base: &'a Record, registry: &Registry, field: &str) -> TestResult<&'a str> {
    let id = registry
        .id(field)
        .ok_or("Proton fixture binding field ID absent")?;
    match record::path_value(base.root(), &[id])? {
        Some(ValueRef::Text(value)) => Ok(value),
        _ => Err("Proton fixture persisted binding is not text".into()),
    }
}
