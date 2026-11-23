mod endpoints;
mod identity;
mod persistence;
mod routing_cases;
mod sources;
use persistence::persist;

use super::*;
use catalog_tool::{endpoints as catalog_endpoints, kind_map, overlay, runtime_catalog, smithy};
use inventory_core::{
    catalog::runtime::{CallablePlan, RuntimeCatalog},
    inventory::{Observation, ResourceKey},
    record::Value,
};
use std::{collections::BTreeMap, sync::LazyLock};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
static CATALOG: LazyLock<Result<Vec<u8>, String>> =
    LazyLock::new(|| compile().map_err(|error| error.to_string()));

fn bytes() -> TestResult<&'static [u8]> {
    CATALOG
        .as_ref()
        .map(Vec::as_slice)
        .map_err(|error| error.clone().into())
}

fn compile() -> TestResult<Vec<u8>> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("source");
    let embedded = overlay::embedded()?;
    let mut policies = Vec::new();
    for (service, date, operations) in [
        (
            "s3-control",
            "2018-08-20",
            &[
                "ListAccessPoints",
                "ListRegionalBuckets",
                "ListAccessPointsForDirectoryBuckets",
                "ListMultiRegionAccessPoints",
            ][..],
        ),
        (
            "transfer",
            "2018-11-05",
            &["ListServers", "ListAgreements", "DescribeAgreement"][..],
        ),
        ("outposts", "2019-12-03", &["ListOutposts"][..]),
        (
            "route-53",
            "2013-04-01",
            &[
                "ListHostedZones",
                "GetHostedZone",
                "ListReusableDelegationSets",
                "GetReusableDelegationSet",
                "ListTrafficPolicyInstancesByHostedZone",
            ][..],
        ),
        (
            "sns",
            "2010-03-31",
            &[
                "ListTopics",
                "ListSubscriptions",
                "GetSubscriptionAttributes",
                "GetTopicAttributes",
            ][..],
        ),
        (
            "sqs",
            "2012-11-05",
            &["ListQueues", "GetQueueAttributes"][..],
        ),
        (
            "wafv2",
            "2019-07-29",
            &[
                "ListWebACLs",
                "GetWebACL",
                "ListIPSets",
                "ListRegexPatternSets",
                "ListRuleGroups",
                "ListTagsForResource",
            ][..],
        ),
        (
            "timestream-write",
            "2018-11-01",
            &[
                "DescribeEndpoints",
                "ListDatabases",
                "DescribeDatabase",
                "ListTables",
                "DescribeTable",
            ][..],
        ),
        (
            "timestream-query",
            "2018-11-01",
            &[
                "DescribeEndpoints",
                "ListScheduledQueries",
                "DescribeScheduledQuery",
            ][..],
        ),
    ] {
        crate::pinned_model::install(&source.join("models"), service, date, operations)?;
        let policy = embedded
            .iter()
            .find(|candidate| candidate.service() == service)
            .ok_or("production service absent")?;
        let selected = operations
            .iter()
            .map(|operation| {
                let key = overlay::OperationName::new(operation)?;
                Ok((
                    *operation,
                    policy
                        .operations()
                        .get(&key)
                        .ok_or("production operation absent")?
                        .clone(),
                ))
            })
            .collect::<TestResult<Vec<_>>>()?;
        policies.push(overlay::ServiceOverlay::new(service, selected)?);
    }
    let checked = overlay::check_policies(
        &source,
        &policies,
        &catalog_tool::classes::DenyList::builtin(),
    )?;
    let fields = smithy::Registry::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/field-ids.txt"
    )))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    Ok(runtime_catalog::augment(
        &base,
        &checked,
        &kinds,
        &catalog_endpoints::verified_endpoints(&source)?,
    )?)
}

fn decode(plan: &CallablePlan<'_>, body: &[u8]) -> TestResult<Record> {
    use aws_wire::{
        request::Protocol,
        response::{DecodeLimits, ResponseContext},
    };
    let protocol = match plan.protocol() {
        inventory_core::catalog::runtime::RuntimeProtocol::RestXml => Protocol::RestXml,
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson11 => Protocol::AwsJson11,
        inventory_core::catalog::runtime::RuntimeProtocol::RestJson1 => Protocol::RestJson1,
        inventory_core::catalog::runtime::RuntimeProtocol::AwsJson10 => Protocol::AwsJson10,
        inventory_core::catalog::runtime::RuntimeProtocol::AwsQuery => Protocol::AwsQuery,
        _ => return Err("unexpected production fixture protocol".into()),
    };
    let headers = http::HeaderMap::new();
    let context = ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol,
        body,
        headers: &headers,
        status: http::StatusCode::OK,
        limits: DecodeLimits::default(),
    };
    Ok(aws_wire::response::decode(context)?)
}

fn request_bindings(
    plan: &CallablePlan<'_>,
    fields: &[super::super::input::InputField],
) -> TestResult<Record> {
    let account = AccountId::parse("123456789012")?;
    let region = inventory_core::inventory::Region::Regional(RegionId::parse("us-east-1")?);
    let base = if super::super::input::requires_parent(plan) {
        Record::encode(&Value::Map(BTreeMap::new()))?
    } else {
        super::super::input::root_bindings(plan, Some(&account), Some(&region))?
    };
    Ok(super::super::input::merge_input(&base, fields)?)
}

fn request(
    plan: &CallablePlan<'_>,
    fields: &[super::super::input::InputField],
) -> TestResult<http::Request<bytes::Bytes>> {
    let bindings = request_bindings(plan, fields)?;
    let region = inventory_core::inventory::Region::Regional(RegionId::parse("us-east-1")?);
    let region = super::super::execution::routing::request_region(
        plan,
        &region,
        &RegionId::parse("us-east-1")?,
        None,
    )?;
    let endpoint = super::super::execution::routing::resolve(plan, &region)?;
    Ok(super::super::request::encode(
        plan,
        &endpoint.url,
        &bindings,
        &[],
    )?)
}

fn observations_with_request(
    plan: &CallablePlan<'_>,
    rows: Vec<Record>,
    request: &Record,
) -> TestResult<Vec<Observation>> {
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    rows.into_iter()
        .map(|row| {
            let row = super::super::resource_projection::project(plan, &row)?;
            let key = ResourceKey {
                account: account.clone(),
                region: inventory_core::inventory::Region::Regional(region.clone()),
                kind: plan.kind().clone(),
                id: resource_id_with_context(
                    &row,
                    plan.identity(),
                    Some(IdentityContext {
                        account: &account,
                        region: &region,
                    }),
                    &inventory_core::inventory::DiscoveryContext::empty(),
                    request,
                )?,
            };
            Ok(Observation::new(key, row, plan.hash_exclusions(), None)?
                .with_facet_precedence(plan.facet_precedence()))
        })
        .collect()
}



fn child_fields<'catalog>(
    catalog: &RuntimeCatalog<'catalog>,
    parent: &CallablePlan<'catalog>,
    child: &CallablePlan<'catalog>,
    row: &Record,
) -> TestResult<Vec<super::super::input::InputField>> {
    child_fields_optional(catalog, parent, child, row)?
        .ok_or_else(|| "required compiled child binding absent".into())
}

fn child_fields_optional<'catalog>(
    catalog: &RuntimeCatalog<'catalog>,
    parent: &CallablePlan<'catalog>,
    child: &CallablePlan<'catalog>,
    row: &Record,
) -> TestResult<Option<Vec<super::super::input::InputField>>> {
    use super::super::execution::page::child::{Context, RowContext, resolve_binding};
    use inventory_core::inventory::{DiscoveryContext, OperationId, Region, ScopeKey, ServiceId};
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    let sweep = inventory_engine::inventory::SweepId::new("qstz-bindings".to_owned())?;
    let scope = ScopeKey::new(
        account.clone(),
        Region::Regional(region.clone()),
        ServiceId::parse(parent.service_name())?,
        OperationId::parse(parent.operation_name())?,
        None,
    )?;
    let ancestry = DiscoveryContext::empty();
    let consumers = super::super::execution::page::consumer_index::ConsumerIndex::build(catalog)?;
    let context = Context::new(parent, &sweep, &account, &region, &consumers);
    let row = RowContext {
        row,
        parent: None,
        scope: &scope,
        ancestry: &ancestry,
    };
    let mut fields = Vec::new();
    for binding in child.bindings() {
        let (_, value) = resolve_binding(&context, &region, &row, &binding.source)?;
        let Some(value) = value else {
            return Ok(None);
        };
        fields.push(super::super::input::InputField {
            path: binding.path.clone(),
            value,
        });
    }
    Ok(Some(fields))
}

