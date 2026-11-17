use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

pub(super) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#ListPortals"},{"target":"n#GetPortal"},{"target":"n#GetPortalServiceProviderMetadata"},{"target":"n#ListBrowserSettings"},{"target":"n#ListUserSettings"}],"traits":{"aws.api#service":{"sdkId":"WorkSpaces Web","endpointPrefix":"workspaces-web","arnNamespace":"workspaces-web"},"aws.auth#sigv4":{"name":"workspaces-web"},"aws.protocols#restJson1":{}}},
"n#ListPortals":{"type":"operation","input":{"target":"n#Empty"},"output":{"target":"n#ListPortalsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/portals"}}},
"n#GetPortal":{"type":"operation","input":{"target":"n#PortalInput"},"output":{"target":"n#GetPortalOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/portal/{portalArn}"}}},
"n#GetPortalServiceProviderMetadata":{"type":"operation","input":{"target":"n#PortalInput"},"output":{"target":"n#MetadataOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/portal/{portalArn}/metadata"}}},
"n#ListBrowserSettings":{"type":"operation","input":{"target":"n#Empty"},"output":{"target":"n#ListBrowserSettingsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/browserSettings"}}},
"n#ListUserSettings":{"type":"operation","input":{"target":"n#Empty"},"output":{"target":"n#ListUserSettingsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/userSettings"}}},
"n#Empty":{"type":"structure","members":{}},
"n#PortalInput":{"type":"structure","members":{"portalArn":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#httpLabel":{}}},"edition":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"edition"}}}},
"n#ListPortalsOutput":{"type":"structure","members":{"portals":{"target":"n#Portals"}}},
"n#Portals":{"type":"list","member":{"target":"n#Portal"}},
"n#GetPortalOutput":{"type":"structure","members":{"portal":{"target":"n#Portal"}}},
"n#Portal":{"type":"structure","members":{"portalArn":{"target":"smithy.api#String"},"displayName":{"target":"smithy.api#String"},"browserSettingsArn":{"target":"smithy.api#String"}}},
"n#MetadataOutput":{"type":"structure","members":{"portalArn":{"target":"smithy.api#String"},"serviceProviderSamlMetadata":{"target":"smithy.api#String"},"zzMetadataBrowserSettingsArn":{"target":"smithy.api#String"}}},
"n#ListBrowserSettingsOutput":{"type":"structure","members":{"browserSettings":{"target":"n#BrowserSettingsList"}}},
"n#BrowserSettingsList":{"type":"list","member":{"target":"n#BrowserSettings"}},
"n#BrowserSettings":{"type":"structure","members":{"browserSettingsArn":{"target":"smithy.api#String"}}},
"n#ListUserSettingsOutput":{"type":"structure","members":{"userSettings":{"target":"n#UserSettingsList"}}},
"n#UserSettingsList":{"type":"list","member":{"target":"n#UserSettings"}},
"n#UserSettings":{"type":"structure","members":{"userSettingsArn":{"target":"smithy.api#String"},"browserSettingsArn":{"target":"smithy.api#String"}}}
}}"#;

pub(super) fn compiled() -> TestResult<Vec<u8>> {
    compile(false, false, false)
}

pub(super) fn compiled_candidates() -> TestResult<Vec<u8>> {
    compile(true, false, false)
}

pub(super) fn compiled_competing() -> TestResult<Vec<u8>> {
    compile(false, true, false)
}

/// GetPortal fans out into two reviewed branches that own the same canonical resource.
pub(super) fn compiled_branched() -> TestResult<Vec<u8>> {
    compile(false, false, true)
}

fn compile(candidates: bool, secondary: bool, branched: bool) -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let models = source.join("models/workspaces-web/service/2022-01-01");
    fs::create_dir_all(&models)?;
    fs::write(models.join("model.json"), MODEL)?;
    let policies = overlay::ServiceOverlay::new("workspaces-web", vec![
        ("ListPortals", plan("portals", "portalArn", None)?),
        ("GetPortal", get_portal(branched)?),
        ("GetPortalServiceProviderMetadata", plan("$", "portalArn", Some("ListPortals"))?),
        ("ListBrowserSettings", plan("browserSettings", "browserSettingsArn", None)?),
        ("ListUserSettings", plan("userSettings", "userSettingsArn", None)?),
    ])?;
    let checked = overlay::check_policies(&source, &[policies], &catalog_tool::classes::DenyList::builtin())?;
    let fields = smithy::Registry::parse("browserSettings\nbrowserSettingsArn\ndisplayName\nportal\nportalArn\nportals\nserviceProviderSamlMetadata\nuserSettings\nuserSettingsArn\nzzMetadataBrowserSettingsArn\nedition\n")?;
    let base = smithy::build_catalog(&source, &fields, "canonical-facet-fixture")?;
    let primary = kind_map::RelationMapping { source_kind: "AWS::WorkSpacesWeb::Portal", source: kind_map::RelationSource::SelectedOutput(kind_map::RelationPath::Fields("browserSettingsArn")), target_kind: "AWS::WorkSpacesWeb::BrowserSettings", target: kind_map::RelationTarget::SelectedOutput(kind_map::RelationPath::Fields("browserSettingsArn")), relation: "USES_BROWSER_SETTINGS", scope: "account_region", guard: None };
    let mut relations = vec![primary];
    if secondary {
        relations.push(kind_map::RelationMapping { source_kind: "AWS::WorkSpacesWeb::Portal", source: kind_map::RelationSource::SelectedOutput(kind_map::RelationPath::Fields("zzMetadataBrowserSettingsArn")), target_kind: "AWS::WorkSpacesWeb::BrowserSettings", target: kind_map::RelationTarget::SelectedOutput(kind_map::RelationPath::Fields("browserSettingsArn")), relation: "USES_METADATA_SETTINGS", scope: "account_region", guard: None });
    }
    let kinds = kind_map::MappingSource {
        cfn: &[
            kind_map::CfnMapping::Mapped { cfn_type: "AWS::WorkSpacesWeb::Portal", kind: "AWS::WorkSpacesWeb::Portal", display_name_path: Some("displayName") },
            kind_map::CfnMapping::canonical("AWS::WorkSpacesWeb::BrowserSettings"),
            kind_map::CfnMapping::canonical("AWS::WorkSpacesWeb::UserSettings"),
        ],
        operations: &[
            kind_map::OperationMapping { service: "workspaces-web", operation: "ListPortals", kind: "AWS::WorkSpacesWeb::Portal" },
            kind_map::OperationMapping { service: "workspaces-web", operation: "GetPortal", kind: "AWS::WorkSpacesWeb::Portal" },
            kind_map::OperationMapping { service: "workspaces-web", operation: "GetPortalServiceProviderMetadata", kind: "AWS::WorkSpacesWeb::Portal" },
            kind_map::OperationMapping { service: "workspaces-web", operation: "ListBrowserSettings", kind: "AWS::WorkSpacesWeb::BrowserSettings" },
            kind_map::OperationMapping { service: "workspaces-web", operation: "ListUserSettings", kind: "AWS::WorkSpacesWeb::UserSettings" },
        ],
        relations: &relations,
        relation_targets: if candidates { &[kind_map::RelationTargetsMapping {
            source_kind: "AWS::WorkSpacesWeb::Portal", source: kind_map::RelationSource::SelectedOutput(kind_map::RelationPath::Fields("browserSettingsArn")), relation: "USES_BROWSER_SETTINGS",
            target_kinds: &["AWS::WorkSpacesWeb::BrowserSettings", "AWS::WorkSpacesWeb::UserSettings"],
        }] } else { &[] },
        ..kind_map::MappingSource::default()
    }.compile(&names::Exceptions::none())?;
    Ok(runtime_catalog::augment(&base, &checked, &kinds, &BTreeMap::from([("workspaces-web".to_owned(), endpoint()?)]))?)
}

fn plan(items: &str, identity: &str, parent: Option<&str>) -> TestResult<overlay::OperationOverlay> {
    let mut plan = overlay::OperationOverlay::default();
    plan.class = Some(if parent.is_some() { overlay::OverlayClass::Expand } else { overlay::OverlayClass::Enumerate });
    plan.scope = Some(overlay::Scope::Region);
    plan.items = Some(overlay::ItemSelector::parse(items)?);
    plan.identity = Some(overlay::Identity::Arn(overlay::MemberPath::selected(identity)?));
    plan.paginator = Some(overlay::Paginator::None);
    plan.queryable = [identity, "displayName", "browserSettingsArn", "serviceProviderSamlMetadata", "zzMetadataBrowserSettingsArn"].into_iter()
        .filter(|field| *field == identity || match items { "portal" | "portals" => matches!(*field, "displayName" | "browserSettingsArn"), "userSettings" => *field == "browserSettingsArn", "$" => matches!(*field, "serviceProviderSamlMetadata" | "zzMetadataBrowserSettingsArn"), _ => false })
        .map(overlay::MemberPath::selected).collect::<Result<std::collections::BTreeSet<_>, _>>()?.into_iter().collect();
    if let Some(parent) = parent {
        plan.bind.insert(overlay::MemberPath::new("portalArn")?, overlay::Binding::From { operation: overlay::OperationName::new(parent)?, path: overlay::MemberPath::selected("portalArn")? });
    }
    Ok(plan)
}

fn endpoint() -> TestResult<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"workspaces-web","signingRegion":"us-east-1"}]});
    let rules = endpoints::RuleSet::parse(Path::new("facet-endpoint.json"), &json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://workspaces-web.test","properties":properties}}]}))?;
    let cases: Vec<endpoints::Case> = serde_json::from_value(json!([{"params":{"Region":"us-east-1"},"expect":{"endpoint":{"url":"https://workspaces-web.test","properties":properties}}}]))?;
    rules.run_cases(&cases)?.verified_endpoint(Some("workspaces-web")).map_err(|error| format!("endpoint fixture: {error:?}").into())
}

fn get_portal(branched: bool) -> TestResult<overlay::OperationOverlay> {
    let mut plan = plan("portal", "portalArn", Some("ListPortals"))?;
    if branched {
        plan.branches = ["alpha", "omega"].into_iter().map(|edition| Ok(overlay::ExecutionBranch {
            discriminator: edition.to_owned(),
            fixed: BTreeMap::from([(overlay::MemberPath::new("edition")?, overlay::FixedValue::String(edition.to_owned()))]),
            bind: BTreeMap::new(),
            emission: None,
            row_policy: None,
            scope: overlay::Scope::Region,
            request_region: None,
        })).collect::<TestResult<_>>()?;
    }
    Ok(plan)
}
