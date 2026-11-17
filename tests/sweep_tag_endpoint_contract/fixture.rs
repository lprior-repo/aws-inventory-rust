use super::*;
use catalog_tool::{endpoints, kind_map, names, overlay, runtime_catalog, smithy};
use std::{collections::BTreeMap, process::{Output, Stdio}};
use tokio::process::{Child, Command};

const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"n#Service":{"type":"service","version":"1","operations":[{"target":"n#GetCallerIdentity"},{"target":"n#DescribeRegions"},{"target":"n#ListA"},{"target":"n#ListZ"},{"target":"n#ListTagsForResource"}],"traits":{"aws.api#service":{"sdkId":"Taginventory","endpointPrefix":"taginventory","arnNamespace":"taginventory"},"aws.auth#sigv4":{"name":"taginventory"},"aws.protocols#restJson1":{}}},
"n#GetCallerIdentity":{"type":"operation","output":{"target":"n#IdentityOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/identity"}}},
"n#IdentityOutput":{"type":"structure","members":{"Account":{"target":"smithy.api#String"}}},
"n#DescribeRegions":{"type":"operation","output":{"target":"n#RegionsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/regions"}}},
"n#RegionsOutput":{"type":"structure","members":{"Regions":{"target":"n#RegionsList"}}},
"n#RegionsList":{"type":"list","member":{"target":"n#Region"}},
"n#Region":{"type":"structure","members":{"RegionName":{"target":"smithy.api#String"}}},
"n#ListA":{"type":"operation","output":{"target":"n#ItemsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/others"}}},
"n#ListZ":{"type":"operation","output":{"target":"n#ItemsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/items"}}},
"n#ItemsOutput":{"type":"structure","members":{"Items":{"target":"n#Items"}}},
"n#Items":{"type":"list","member":{"target":"n#Item"}},
"n#Item":{"type":"structure","members":{"Arn":{"target":"smithy.api#String"},"Tags":{"target":"n#TagMap"}}},
"n#TagMap":{"type":"map","key":{"target":"smithy.api#String"},"value":{"target":"smithy.api#String"}},
"n#ListTagsForResource":{"type":"operation","input":{"target":"n#TagsInput"},"output":{"target":"n#TagsOutput"},"traits":{"smithy.api#http":{"method":"GET","uri":"/tags"}}},
"n#TagsInput":{"type":"structure","members":{"ResourceArn":{"target":"smithy.api#String","traits":{"smithy.api#required":{},"smithy.api#httpQuery":"arn"}},"Marker":{"target":"smithy.api#String","traits":{"smithy.api#httpQuery":"marker"}}}},
"n#TagsOutput":{"type":"structure","members":{"Tags":{"target":"n#TagsList","traits":{"smithy.api#required":{}}},"NextToken":{"target":"smithy.api#String"},"IsTruncated":{"target":"smithy.api#Boolean","traits":{"smithy.api#default":false}}}},
"n#TagsList":{"type":"list","member":{"target":"n#Tag"}},
"n#Tag":{"type":"structure","members":{"Key":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}},"Value":{"target":"smithy.api#String","traits":{"smithy.api#required":{}}}}}
}}"#;

const KINDS: kind_map::MappingSource<'static> = kind_map::MappingSource {
    version: 1, reviewed: "2026-11-15", reviewer: "Tag consumer regression", default_unmapped_reason: None,
    cfn: &[], relations: &[], parent_relations: &[], compound_relations: &[], additional_emissions: &[], relation_targets: &[],
    api_only: &[
        kind_map::ApiOnlyMapping { kind: "AWS::ApiOnly::TagAccount", reason: "Bootstrap account" },
        kind_map::ApiOnlyMapping { kind: "AWS::ApiOnly::TagRegion", reason: "Bootstrap region" },
        kind_map::ApiOnlyMapping { kind: PARENT, reason: "Owner metadata" },
        kind_map::ApiOnlyMapping { kind: OTHER, reason: "Different kind with colliding ID" },
    ],
    operations: &[
        kind_map::OperationMapping { service: "taginventory", operation: "GetCallerIdentity", kind: "AWS::ApiOnly::TagAccount" },
        kind_map::OperationMapping { service: "taginventory", operation: "DescribeRegions", kind: "AWS::ApiOnly::TagRegion" },
        kind_map::OperationMapping { service: "taginventory", operation: "ListZ", kind: PARENT },
        kind_map::OperationMapping { service: "taginventory", operation: "ListA", kind: OTHER },
    ],
    bootstraps: &[
        kind_map::BootstrapMapping::AccountIdentity { service: "taginventory", operation: "GetCallerIdentity", account_path: "Account" },
        kind_map::BootstrapMapping::RegionDiscovery { service: "taginventory", operation: "DescribeRegions", regions_path: "Regions", region_name_path: "RegionName", opt_in_status_path: None },
    ],
};

fn policy(map: bool) -> Result<overlay::ServiceOverlay> {
    let path = overlay::MemberPath::new;
    let plan = |scope, items, identity| -> Result<overlay::OperationOverlay> {
        let mut operation = overlay::OperationOverlay::default();
        operation.class = Some(overlay::OverlayClass::Enumerate);
        operation.scope = Some(scope);
        operation.items = Some(overlay::ItemSelector::parse(items)?);
        operation.identity = Some(overlay::Identity::Key(vec![overlay::MemberPath::selected(identity)?]));
        operation.paginator = Some(overlay::Paginator::None);
        Ok(operation)
    };
    let mut parent = plan(overlay::Scope::Region, "Items", "Arn")?;
    parent.tags = Some(path("Tags")?);
    let selector = if map { overlay::TagEndpointSelector::Map { path: path("Tags")? } } else { overlay::TagEndpointSelector::List { path: path("Tags")?, key: path("Key")?, value: path("Value")? } };
    let tags = overlay::TagEndpoint { parents: vec![overlay::TagEndpointParent { operation: overlay::OperationName::new("ListZ")?, request: overlay::TagEndpointRequest::Member { input: path("ResourceArn")?, parent: path("Arn")? } }], selector }.operation(overlay::Scope::Region, overlay::Paginator::Tokens { pairs: vec![overlay::TokenPair::new(path("Marker")?, path("NextToken")?)], truncated: Some(path("IsTruncated")?) })?;
    Ok(overlay::ServiceOverlay::new("taginventory", vec![("GetCallerIdentity", plan(overlay::Scope::Account, "$", "Account")?), ("DescribeRegions", plan(overlay::Scope::Region, "Regions", "RegionName")?), ("ListA", parent.clone()), ("ListZ", parent), ("ListTagsForResource", tags)])?)
}

pub(super) struct Fixture { pub root: tempfile::TempDir, pub database: PathBuf, catalog: PathBuf, credentials: PathBuf, config: PathBuf }
impl Fixture {
    pub fn create(map: bool) -> Result<Self> {
        let root = tempfile::tempdir()?;
        let source = root.path().join("source");
        let models = source.join("models/taginventory/service/2026-01-01");
        std::fs::create_dir_all(&models)?;
        let model = if map { MODEL.replace("\"Tags\":{\"target\":\"n#TagsList\"", "\"Tags\":{\"target\":\"n#TagMap\"") } else { MODEL.to_owned() };
        std::fs::write(models.join("model.json"), model)?;
        let checked = overlay::check_policies(&source, &[policy(map)?], &catalog_tool::classes::DenyList::builtin())?;
        let registry = smithy::Registry::parse("Account\nArn\nIsTruncated\nItems\nKey\nMarker\nNextToken\nRegionName\nRegions\nResourceArn\nTags\nValue\n")?;
        let base = smithy::build_catalog(&source, &registry, "signed-tag-consumer")?;
        let kinds = KINDS.compile(&names::Exceptions::none())?;
        let endpoints = BTreeMap::from([("taginventory".to_owned(), endpoint()?)]);
        let bytes = runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?;
        let catalog = root.path().join("fixture.catalog");
        std::fs::write(&catalog, bytes)?;
        let credentials = root.path().join("credentials");
        let config = root.path().join("config");
        std::fs::write(&credentials, "[tags]\naws_access_key_id=AKIATAGCONTRACT\naws_secret_access_key=local-only-tag-secret\n")?;
        std::fs::write(&config, "[profile tags]\nregion=us-east-1\n")?;
        let database = root.path().join("inventory.sqlite");
        Ok(Self { root, database, catalog, credentials, config })
    }
    pub fn spawn(&self, peer: &peer::Peer, resume: bool, max_pages: u32) -> Result<Child> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_aws-inventory"));
        command.env_clear().env("HOME", self.root.path()).env("AWS_CONFIG_FILE", &self.config).env("AWS_SHARED_CREDENTIALS_FILE", &self.credentials).env("AWS_EC2_METADATA_DISABLED", "true")
            .args(["sweep", "--database"]).arg(&self.database).arg("--catalog").arg(&self.catalog).args(["--profile", "tags", "--region", REGION, "--endpoint"]).arg(&peer.endpoint).arg("--ca-bundle").arg(&peer.ca).arg("--max-pages").arg(max_pages.to_string()).args(["--concurrency", "1"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
        if resume { command.arg("--resume"); }
        command.spawn().context("launching signed tag collector")
    }
    pub async fn collect(&self, peer: &peer::Peer, resume: bool, max_pages: u32) -> Result<Output> {
        let mut child = self.spawn(peer, resume, max_pages)?;
        match tokio::time::timeout(DEADLINE, child.wait()).await {
            Ok(status) => { status?; }
            Err(error) => { child.kill().await?; return Err(error.into()); }
        }
        Ok(tokio::time::timeout(DEADLINE, child.wait_with_output()).await??)
    }
}
fn endpoint() -> Result<endpoints::VerifiedEndpoint> {
    let properties = json!({"authSchemes":[{"name":"sigv4","signingName":"taginventory","signingRegion":REGION}]});
    let document = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},"rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":"https://taginventory.invalid","properties":properties}}]});
    let rules = endpoints::RuleSet::parse(Path::new("tag-endpoint.json"), &document)?;
    let cases = serde_json::from_value::<Vec<endpoints::Case>>(json!([{"params":{"Region":REGION},"expect":{"endpoint":{"url":"https://taginventory.invalid","properties":properties}}}]))?;
    rules.run_cases(&cases)?.verified_endpoint(Some("taginventory")).map_err(|error| anyhow::anyhow!("fixture endpoint error: {error:?}"))
}
