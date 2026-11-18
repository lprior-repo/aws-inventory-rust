mod endpoints;
mod identity;

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
        crate::sweep_runtime::pinned_model::install(
            &source.join("models"),
            service,
            date,
            operations,
        )?;
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
        "/catalog/field-ids.txt"
    )))?;
    let base = smithy::build_catalog(&source, &fields, "46135f8468cd5cb60d983b97290c9f6661e359c6")?;
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/catalog/cloudcontrol-universal.json"
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

async fn persist(
    plan: &CallablePlan<'_>,
    pages: Vec<(
        &CallablePlan<'_>,
        Option<ResourceKey>,
        Record,
        Vec<Observation>,
    )>,
) -> TestResult<(Vec<(String, String, Vec<u8>)>, Vec<(String, String)>)> {
    use inventory_core::inventory::{OperationId, Region, ScopeKey, ServiceId};
    use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
    let root = tempfile::tempdir()?;
    let database = root.path().join("inventory.db");
    let runtime = RuntimeCatalog::from_bytes(bytes()?)?;
    let store = BinaryStore::create(
        &database,
        SchemaRegistry::new(bytes()?, runtime.schemas().to_vec())?,
    )
    .await?;
    let sweep = SweepId::new("qstz-policy-regression".to_owned())?;
    for (producer, parent, bindings, observations) in pages {
        let scope = ScopeKey::new(
            AccountId::parse("123456789012")?,
            Region::Regional(RegionId::parse("us-east-1")?),
            ServiceId::parse(producer.service_name())?,
            OperationId::parse(producer.operation_name())?,
            parent,
        )?;
        store
            .seed_work(WorkSpec::new(
                sweep.clone(),
                scope,
                None,
                bindings.into_bytes(),
            )?)
            .await?;
        let tags = if let Some(tags) = producer.tags() {
            observations
                .iter()
                .map(|observation| tag_observation(observation.record(), observation.key(), tags))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        let lease = store
            .claim(&sweep, 1000, 10000)
            .await?
            .ok_or("seeded work absent")?;
        store
            .commit_page(PageCommit {
                lease,
                observations,
                edges: Vec::new(),
                unresolved: Vec::new(),
                tags,
                next_token: None,
                continuation_digest: None,
                children: Vec::new(),
                done: true,
                observed_at_ms: 1001,
            })
            .await?;
    }
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&database)
                .read_only(true),
        )
        .await?;
    let resources =
        sqlx::query_as("SELECT kind,id,record FROM resource_facets WHERE kind=?1 ORDER BY id")
            .bind(plan.kind().as_str())
            .fetch_all(&reader)
            .await?;
    let tags = sqlx::query_as("SELECT tag_key,tag_value FROM tags ORDER BY tag_key")
        .fetch_all(&reader)
        .await?;
    reader.close().await;
    Ok((resources, tags))
}

#[tokio::test]
async fn given_all_source_access_points_on_two_pages_when_wire_decoded_then_every_canonical_arn_persists()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let plan = catalog
        .lookup("s3-control", "ListAccessPoints")
        .ok_or("access-point plan absent")?;
    let first_request = request(plan, &[])?;
    assert!(
        first_request
            .uri()
            .query()
            .ok_or("query absent")?
            .split('&')
            .any(|pair| pair == "dataSourceType=ALL")
    );
    assert_eq!(
        first_request.uri().host(),
        Some("123456789012.s3-control.us-east-1.amazonaws.com")
    );
    assert_eq!(
        first_request
            .headers()
            .get("x-amz-account-id")
            .ok_or("account header absent")?,
        "123456789012"
    );
    let first = decode(plan, br#"<ListAccessPointsResult><AccessPointList><AccessPoint><Name>ordinary</Name><NetworkOrigin>Internet</NetworkOrigin><Bucket>bucket</Bucket><AccessPointArn>arn:aws:s3:us-east-1:123456789012:accesspoint/ordinary</AccessPointArn><DataSourceType>S3</DataSourceType></AccessPoint></AccessPointList><NextToken>second-page</NextToken></ListAccessPointsResult>"#)?;
    let token = plan
        .pagination()
        .tokens
        .first()
        .ok_or("compiled cursor absent")?;
    let cursor = path_value(first.root(), &token.response_path)?
        .ok_or("continuation absent")?
        .to_value()?;
    let next_request = request(
        plan,
        &[super::super::input::InputField {
            path: token.request_path.clone(),
            value: cursor,
        }],
    )?;
    assert!(
        next_request
            .uri()
            .query()
            .ok_or("next query absent")?
            .split('&')
            .any(|pair| pair == "nextToken=second-page")
    );
    let second = decode(plan, br#"<ListAccessPointsResult><AccessPointList><AccessPoint><Name>filesystem</Name><NetworkOrigin>VPC</NetworkOrigin><Bucket>filesystem</Bucket><AccessPointArn>arn:aws:s3:us-east-1:123456789012:accesspoint/filesystem</AccessPointArn><DataSourceType>FSX_OPENZFS</DataSourceType></AccessPoint></AccessPointList></ListAccessPointsResult>"#)?;
    let mut rows = item_records(&first, plan.items())?;
    rows.extend(item_records(&second, plan.items())?);
    let bindings = request_bindings(plan, &[])?;
    let observations = observations_with_request(plan, rows, &bindings)?;
    let (persisted, _) = persist(plan, vec![(plan, None, bindings, observations)]).await?;
    assert_eq!(
        persisted
            .iter()
            .map(|(kind, id, _)| (kind.as_str(), id.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                "AWS::S3::AccessPoint",
                "arn:aws:s3:us-east-1:123456789012:accesspoint/filesystem"
            ),
            (
                "AWS::S3::AccessPoint",
                "arn:aws:s3:us-east-1:123456789012:accesspoint/ordinary"
            ),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn given_agreements_on_two_servers_when_details_bind_then_exact_pairs_safe_fields_and_tags_persist()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("transfer", "ListAgreements")
        .ok_or("agreement list absent")?;
    let detail = catalog
        .lookup("transfer", "DescribeAgreement")
        .ok_or("agreement detail absent")?;
    let page = decode(list, br#"{"Agreements":[{"Arn":"arn:aws:transfer:us-east-1:123456789012:agreement/s-one/a-one","AgreementId":"a-one","ServerId":"s-one"},{"Arn":"arn:aws:transfer:us-east-1:123456789012:agreement/s-two/a-two","AgreementId":"a-two","ServerId":"s-two"}],"NextToken":"next"}"#)?;
    assert!(
        path_value(
            page.root(),
            &list
                .pagination()
                .tokens
                .first()
                .ok_or("list token absent")?
                .response_path
        )?
        .is_some()
    );
    let mut pairs = Vec::new();
    let mut first_bindings = None;
    for row in item_records(&page, list.items())? {
        let fields = child_fields(&catalog, list, detail, &row)?;
        let encoded = request(detail, &fields)?;
        if first_bindings.is_none() {
            first_bindings = Some(request_bindings(detail, &fields)?);
        }
        let body: serde_json::Value = serde_json::from_slice(encoded.body())?;
        pairs.push((
            body.get("ServerId")
                .and_then(serde_json::Value::as_str)
                .ok_or("server absent")?
                .to_owned(),
            body.get("AgreementId")
                .and_then(serde_json::Value::as_str)
                .ok_or("agreement absent")?
                .to_owned(),
        ));
    }
    assert_eq!(
        pairs,
        [
            ("s-one".to_owned(), "a-one".to_owned()),
            ("s-two".to_owned(), "a-two".to_owned())
        ]
    );
    let response = decode(detail, br#"{"Agreement":{"Arn":"arn:aws:transfer:us-east-1:123456789012:agreement/s-one/a-one","AgreementId":"a-one","ServerId":"s-one","BaseDirectory":"/bucket/inbound","AccessRole":"arn:aws:iam::123456789012:role/transfer","Tags":[{"Key":"team","Value":"inventory"}]}}"#)?;
    let rows = item_records(&response, detail.items())?;
    let expected = rows.first().ok_or("detail row absent")?.as_bytes().to_vec();
    let bindings = first_bindings.ok_or("detail request absent")?;
    let observations = observations_with_request(detail, rows, &bindings)?;
    let (persisted, tags) = persist(detail, vec![(detail, None, bindings, observations)]).await?;
    let (kind, id, record) = persisted.first().ok_or("detail not persisted")?;
    assert_eq!(persisted.len(), 1);
    assert_eq!(kind, "AWS::Transfer::Agreement");
    assert_eq!(
        id,
        "arn:aws:transfer:us-east-1:123456789012:agreement/s-one/a-one"
    );
    assert_eq!(record, &expected);
    assert_eq!(tags, [("team".to_owned(), "inventory".to_owned())]);
    Ok(())
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

#[tokio::test]
async fn given_two_outposts_when_bucket_pages_fan_out_then_wire_signing_headers_and_bucket_arns_are_correct()
-> TestResult {
    use aws_wire::sign::{self, Body, Credentials, PathMode, PayloadHeader, Scope, SigningName};
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let parent = catalog
        .lookup("outposts", "ListOutposts")
        .ok_or("Outposts parent absent")?;
    let plan = catalog
        .lookup("s3-control", "ListRegionalBuckets")
        .ok_or("regional buckets absent")?;
    let outposts = decode(parent, br#"{"Outposts":[{"OutpostId":"op-01234567890123456","OutpostArn":"arn:aws:outposts:us-east-1:123456789012:outpost/op-01234567890123456"},{"OutpostId":"op-fedcba98765432100","OutpostArn":"arn:aws:outposts:us-east-1:123456789012:outpost/op-fedcba98765432100"}]}"#)?;
    let parent_bindings = request_bindings(parent, &[])?;
    let owners = observations_with_request(
        parent,
        item_records(&outposts, parent.items())?,
        &parent_bindings,
    )?;
    let mut pages = Vec::new();
    let mut expected = Vec::new();
    for owner in &owners {
        let row = owner.record();
        let fields = child_fields(&catalog, parent, plan, row)?;
        let first = request(plan, &fields)?;
        assert_eq!(
            first.uri().host(),
            Some("s3-outposts.us-east-1.amazonaws.com")
        );
        assert_eq!(
            first
                .headers()
                .get("x-amz-account-id")
                .ok_or("account header absent")?,
            "123456789012"
        );
        let outpost = first
            .headers()
            .get("x-amz-outpost-id")
            .ok_or("Outpost header absent")?
            .to_str()?;
        assert!(matches!(
            outpost,
            "op-01234567890123456" | "op-fedcba98765432100"
        ));
        let endpoint =
            super::super::execution::routing::resolve(plan, &RegionId::parse("us-east-1")?)?;
        let scope = Scope {
            region: sign::Region::new(&endpoint.signing_region)?,
            name: SigningName::new(&endpoint.signing_name)?,
        };
        let uri = first.uri().to_string();
        let signed = sign::sign(
            &Credentials::new("A", "secret", None)?,
            &scope,
            std::time::SystemTime::UNIX_EPOCH,
            sign::Request {
                method: first.method(),
                uri: &uri,
                headers: first.headers(),
                body: Body::Bytes(first.body()),
                payload_header: PayloadHeader::Omit,
            },
            PathMode::Standard,
        )?;
        assert!(
            signed
                .get("authorization")
                .ok_or("authorization absent")?
                .to_str()?
                .contains("/us-east-1/s3-outposts/aws4_request,")
        );
        let mut rows = Vec::new();
        for page in 1..=2 {
            let arn = format!(
                "arn:aws:s3-outposts:us-east-1:123456789012:outpost/{outpost}/bucket/bucket-{page}"
            );
            let token = if page == 1 {
                "<NextToken>page-two</NextToken>"
            } else {
                ""
            };
            let body = format!(
                "<ListRegionalBucketsResult><RegionalBucketList><RegionalBucket><Bucket>bucket-{page}</Bucket><BucketArn>{arn}</BucketArn><OutpostId>{outpost}</OutpostId><PublicAccessBlockEnabled>false</PublicAccessBlockEnabled><CreationDate>2026-01-01T00:00:00Z</CreationDate></RegionalBucket></RegionalBucketList>{token}</ListRegionalBucketsResult>"
            );
            let response = decode(plan, body.as_bytes())?;
            if page == 1 {
                let token = plan
                    .pagination()
                    .tokens
                    .first()
                    .ok_or("regional bucket cursor absent")?;
                let value = path_value(response.root(), &token.response_path)?
                    .ok_or("bucket response cursor absent")?
                    .to_value()?;
                let mut second_fields = child_fields(&catalog, parent, plan, row)?;
                second_fields.push(super::super::input::InputField {
                    path: token.request_path.clone(),
                    value,
                });
                let second = request(plan, &second_fields)?;
                assert_eq!(
                    second.headers().get("x-amz-outpost-id"),
                    first.headers().get("x-amz-outpost-id")
                );
                assert!(
                    second
                        .uri()
                        .query()
                        .ok_or("bucket page query absent")?
                        .split('&')
                        .any(|pair| pair == "nextToken=page-two")
                );
            }
            rows.extend(item_records(&response, plan.items())?);
            expected.push(arn);
        }
        let bindings = request_bindings(plan, &fields)?;
        let observations = observations_with_request(plan, rows, &bindings)?;
        pages.push((plan, Some(owner.key().clone()), bindings, observations));
    }
    pages.insert(0, (parent, None, parent_bindings, owners));
    assert_eq!(expected.len(), 4);
    expected.sort();
    let (persisted, _) = persist(plan, pages).await?;
    assert_eq!(
        persisted
            .iter()
            .map(|(_, id, _)| id.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(
        persisted
            .iter()
            .all(|(kind, _, _)| kind == plan.kind().as_str())
    );
    Ok(())
}

#[test]
fn given_optional_route53_id_prefixes_when_child_requests_bind_then_only_request_ids_normalize()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    for (list, get, field, wrapper, segment, prefix) in [
        (
            "ListHostedZones",
            "GetHostedZone",
            "HostedZones",
            "HostedZone",
            "hostedzone",
            "/hostedzone/",
        ),
        (
            "ListReusableDelegationSets",
            "GetReusableDelegationSet",
            "DelegationSets",
            "DelegationSet",
            "delegationset",
            "/delegationset/",
        ),
    ] {
        let parent = catalog
            .lookup("route-53", list)
            .ok_or("Route53 list absent")?;
        let child = catalog
            .lookup("route-53", get)
            .ok_or("Route53 detail absent")?;
        for identifier in [format!("{prefix}Z123456"), "Z123456".to_owned()] {
            let body = format!(
                "<{list}Response><{field}><{wrapper}><Id>{identifier}</Id></{wrapper}></{field}></{list}Response>"
            );
            let response = decode(parent, body.as_bytes())?;
            let row = item_records(&response, parent.items())?
                .into_iter()
                .next()
                .ok_or("Route53 row absent")?;
            let original_id = resource_id(&row, parent.identity(), None)?;
            assert_eq!(original_id.as_str(), identifier);
            let encoded = request(child, &child_fields(&catalog, parent, child, &row)?)?;
            assert_eq!(
                encoded.uri().path(),
                format!("/2013-04-01/{segment}/Z123456")
            );
            assert!(!encoded.uri().path().contains("%2F"));
            assert_eq!(resource_id(&row, parent.identity(), None)?, original_id);
            if list == "ListHostedZones" {
                let traffic = catalog
                    .lookup("route-53", "ListTrafficPolicyInstancesByHostedZone")
                    .ok_or("zone traffic instances absent")?;
                let encoded = request(traffic, &child_fields(&catalog, parent, traffic, &row)?)?;
                assert!(
                    encoded
                        .uri()
                        .query()
                        .ok_or("zone traffic query absent")?
                        .split('&')
                        .any(|pair| pair == "id=Z123456")
                );
                assert!(!encoded.uri().to_string().contains("%2F"));
            }
        }
        for identifier in [
            "".to_owned(),
            prefix.to_owned(),
            format!("{prefix}/Z123456"),
            "/wrong/Z123456".to_owned(),
        ] {
            let body = format!(
                "<{list}Response><{field}><{wrapper}><Id>{identifier}</Id></{wrapper}></{field}></{list}Response>"
            );
            let response = decode(parent, body.as_bytes())?;
            let row = item_records(&response, parent.items())?
                .into_iter()
                .next()
                .ok_or("malformed Route53 row absent")?;
            assert!(
                child_fields(&catalog, parent, child, &row).is_err(),
                "malformed identifier was accepted: {identifier}"
            );
        }
    }
    Ok(())
}

#[test]
fn given_s3_global_and_directory_buckets_when_routes_resolve_then_request_destinations_are_operation_specific()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let mrap = catalog
        .lookup("s3-control", "ListMultiRegionAccessPoints")
        .ok_or("MRAP absent")?;
    let scope = inventory_core::inventory::Region::Global;
    let request_region = super::super::execution::routing::request_region(
        mrap,
        &scope,
        &RegionId::parse("eu-west-1")?,
        None,
    )?;
    assert_eq!(request_region.as_str(), "us-west-2");
    assert_eq!(
        request(mrap, &[])?.uri().host(),
        Some("123456789012.s3-control.us-west-2.amazonaws.com")
    );
    let directory = catalog
        .lookup("s3-control", "ListAccessPointsForDirectoryBuckets")
        .ok_or("directory access points absent")?;
    let dir_req = request(directory, &[])?;
    assert_eq!(
        dir_req.uri().host(),
        Some("s3express-control.us-east-1.amazonaws.com")
    );
    assert_eq!(
        dir_req
            .headers()
            .get("x-amz-account-id")
            .ok_or("account header absent")?,
        "123456789012"
    );
    assert_eq!(
        super::super::execution::routing::resolve(directory, &RegionId::parse("us-east-1")?)?
            .signing_name,
        "s3express"
    );
    Ok(())
}
