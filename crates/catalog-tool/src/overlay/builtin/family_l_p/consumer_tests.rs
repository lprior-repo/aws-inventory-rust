use super::overlays;
use crate::{classes::DenyList, endpoints, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::{
        BindingSource, BindingValue, IdentityPlan, ItemPlan, RuntimeCatalog, TagPlan,
    },
    record::{FieldId, Key},
};
use serde_json::json;
use std::{collections::BTreeMap, error::Error, fs, path::Path};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn compile_pinned(
    service: &str,
    version: &str,
    operations: &[&str],
    kind: &str,
) -> TestResult<Vec<u8>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = tempfile::tempdir()?;
    let relative = format!("models/{service}/service/{version}/{service}-{version}.json");
    let destination = source.path().join(&relative);
    fs::create_dir_all(destination.parent().ok_or("model parent missing")?)?;
    fs::copy(root.join(".local/aws-models").join(relative), &destination)?;
    // The pinned model is a complete service, so submit its complete reviewed policy.
    // A selected-operation overlay leaves unrelated candidates without a disposition.
    // Keep built-in deny enforcement active at the same public checker boundary.
    let policies = [overlays()?
        .into_iter()
        .find(|policy| policy.service() == service)
        .ok_or("pinned service policy missing")?];
    let checked = overlay::check_policies(source.path(), &policies, &DenyList::builtin())?;
    let registry =
        smithy::Registry::parse(&fs::read_to_string(root.join("catalog/field-ids.txt"))?)?;
    let base = smithy::build_catalog(source.path(), &registry, "lp-pinned-regression")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    let endpoints = BTreeMap::from([(service.to_owned(), fixture_endpoint(service)?)]);
    let bytes = runtime_catalog::augment(&base, &checked, &kinds, &endpoints)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let first = operations.first().ok_or("fixture needs an operation")?;
    assert_eq!(
        runtime
            .lookup(service, first)
            .ok_or("compiled operation absent")?
            .kind()
            .as_str(),
        kind
    );
    Ok(bytes)
}

fn fixture_endpoint(service: &str) -> TestResult<endpoints::VerifiedEndpoint> {
    // Transport evidence is deterministic and local; model/policy/schema compilation remains real.
    let url = format!("https://{service}.test");
    let properties =
        json!({"authSchemes":[{"name":"sigv4","signingName":service,"signingRegion":"us-east-1"}]});
    let rules = json!({"version":"1.0","parameters":{"Region":{"type":"string"}},
        "rules":[{"type":"endpoint","conditions":[],"endpoint":{"url":url,"properties":properties}}]});
    let cases =
        serde_json::from_value::<Vec<endpoints::Case>>(json!([{"params":{"Region":"us-east-1"},
        "expect":{"endpoint":{"url":url,"properties":properties}}}]))?;
    let parsed = endpoints::RuleSet::parse(Path::new("lp-regression-endpoint.json"), &rules)?;
    parsed
        .run_cases(&cases)?
        .verified_endpoint(Some(service))
        .map_err(|failure| format!("fixture endpoint failed: {failure:?}").into())
}

fn field(catalog: &RuntimeCatalog<'_>, service: &str, member: &str) -> TestResult<FieldId> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/field-ids.txt");
    let registry = smithy::Registry::parse(&fs::read_to_string(path)?)?;
    let id = registry
        .id(member)
        .ok_or_else(|| format!("field missing: {member}"))?;
    let plan = catalog
        .plans()
        .iter()
        .find(|plan| plan.service_name() == service)
        .ok_or("service plan missing")?;
    assert_eq!(plan.service().field_name(id)?, member);
    Ok(id)
}

#[test]
fn given_pinpoint_scalar_names_when_pinned_pair_compiles_then_detail_keeps_root_identity_and_tags()
-> TestResult {
    // Given: the real Smithy scalar name list and singleton metadata response.
    let bytes = compile_pinned(
        "pinpoint-email",
        "2018-07-26",
        &["ListConfigurationSets", "GetConfigurationSet"],
        "AWS::PinpointEmail::ConfigurationSet",
    )?;
    // When: the compiler's numeric artifact is consumed by the production runtime reader.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("pinpoint-email", "ListConfigurationSets")
        .ok_or("list missing")?;
    let detail = runtime
        .lookup("pinpoint-email", "GetConfigurationSet")
        .ok_or("detail missing")?;
    let name = field(&runtime, "pinpoint-email", "ConfigurationSetName")?;
    let tags = field(&runtime, "pinpoint-email", "Tags")?;
    // Then: list self and detail member encode the same single-component name, not a scalar selector.
    assert_eq!(list.kind(), detail.kind());
    assert_eq!(
        list.identity(),
        &IdentityPlan::Key(vec![Box::<[FieldId]>::default()].into_boxed_slice())
    );
    assert_eq!(detail.items(), &ItemPlan::Root);
    assert_eq!(
        detail.identity(),
        &IdentityPlan::Key(vec![vec![name].into_boxed_slice()].into_boxed_slice())
    );
    assert!(matches!(detail.tags(), Some(TagPlan::List { path, .. }) if path.as_ref() == [tags]));
    assert!(
        matches!(&detail.bindings().first().ok_or("name binding missing")?.source,
        BindingSource::Operation { path, .. } if path.is_empty())
    );
    Ok(())
}

#[test]
fn given_license_summary_when_detail_compiles_then_arn_identity_and_tag_snapshot_share_kind()
-> TestResult {
    // Given: the summary supplies the exact ARN required by the metadata detail request.
    let bytes = compile_pinned(
        "license-manager",
        "2018-08-01",
        &["ListLicenseConfigurations", "GetLicenseConfiguration"],
        "AWS::LicenseManager::LicenseConfiguration",
    )?;
    // When: the pinned shapes and actual overlays compile into consumable runtime plans.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let list = runtime
        .lookup("license-manager", "ListLicenseConfigurations")
        .ok_or("list missing")?;
    let detail = runtime
        .lookup("license-manager", "GetLicenseConfiguration")
        .ok_or("detail missing")?;
    let arn = field(&runtime, "license-manager", "LicenseConfigurationArn")?;
    let tags = field(&runtime, "license-manager", "Tags")?;
    // Then: detail cannot create a second identity or silently omit the inline tag source.
    assert_eq!(list.kind(), detail.kind());
    assert_eq!(
        list.identity(),
        &IdentityPlan::Arn(vec![arn].into_boxed_slice())
    );
    assert_eq!(list.identity(), detail.identity());
    assert_eq!(detail.items(), &ItemPlan::Root);
    assert!(matches!(detail.tags(), Some(TagPlan::List { path, .. }) if path.as_ref() == [tags]));
    assert_eq!(
        detail
            .bindings()
            .first()
            .ok_or("ARN binding missing")?
            .path
            .as_ref(),
        [arn]
    );
    assert!(
        matches!(&detail.bindings().first().ok_or("ARN binding missing")?.source,
        BindingSource::Operation { path, .. } if path.as_ref() == [arn])
    );
    Ok(())
}

#[test]
fn given_mixed_search_engines_when_pinned_opensearch_list_compiles_then_request_retains_all_engines()
-> TestResult {
    // Given: ListDomainNames otherwise returns both OpenSearch and Elasticsearch domains.
    let bytes = compile_pinned(
        "opensearch",
        "2021-01-01",
        &["ListDomainNames"],
        "AWS::OpenSearchService::Domain",
    )?;
    // When: the unfiltered operation is checked and read from the artifact.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = runtime
        .lookup("opensearch", "ListDomainNames")
        .ok_or("list missing")?;
    // Then: no engine binding removes Elasticsearch-engine domains.
    assert_eq!(plan.bindings(), []);
    let name = vec![field(&runtime, "opensearch", "DomainName")?].into_boxed_slice();
    assert_eq!(
        plan.identity(),
        &IdentityPlan::Key(vec![name].into_boxed_slice())
    );
    Ok(())
}

#[test]
fn given_neptune_cluster_discovery_when_compiled_then_engine_filter_is_structured_and_endpoints_require_discovered_cluster()
-> TestResult {
    // Given: Neptune and RDS share the wire DescribeDBClusters API and ARN service spelling.
    let bytes = compile_pinned(
        "neptune",
        "2014-10-31",
        &["DescribeDBClusters", "DescribeDBClusterEndpoints"],
        "AWS::Neptune::DBCluster",
    )?;
    // When: actual pinned structures are checked, encoded and decoded as runtime binding values.
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let clusters = runtime
        .lookup("neptune", "DescribeDBClusters")
        .ok_or("clusters missing")?;
    let filters = field(&runtime, "neptune", "Filters")?;
    let name = field(&runtime, "neptune", "Name")?;
    let values = field(&runtime, "neptune", "Values")?;
    let filter = clusters
        .bindings()
        .iter()
        .find(|binding| binding.path.as_ref() == [filters])
        .ok_or("engine filter absent")?;
    let BindingSource::Fixed(BindingValue::List(entries)) = &filter.source else {
        return Err("Filters is not a modeled structure list".into());
    };
    let [BindingValue::Object(members)] = entries.as_ref() else {
        return Err("exactly one engine filter required".into());
    };
    // Then: numeric modeled structure keys preserve the engine name and its exact value.
    assert_eq!(
        members.iter().cloned().collect::<BTreeMap<_, _>>(),
        BTreeMap::from([
            (Key::Field(name), BindingValue::String("engine".to_owned())),
            (
                Key::Field(values),
                BindingValue::Strings(vec!["neptune".to_owned()].into_boxed_slice())
            ),
        ])
    );
    let endpoints = runtime
        .lookup("neptune", "DescribeDBClusterEndpoints")
        .ok_or("endpoints missing")?;
    let cluster_id = field(&runtime, "neptune", "DBClusterIdentifier")?;
    let binding = endpoints
        .bindings()
        .iter()
        .find(|binding| binding.path.as_ref() == [cluster_id])
        .ok_or("endpoint parent binding absent")?;
    assert_eq!(
        binding.source,
        BindingSource::Operation {
            service: "neptune".into(),
            operation: "DescribeDBClusters".into(),
            path: vec![cluster_id].into_boxed_slice(),
        }
    );
    Ok(())
}
