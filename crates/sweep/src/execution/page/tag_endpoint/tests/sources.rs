use super::*;
use inventory_core::{
    catalog::runtime::RuntimeProtocol,
    inventory::{ResourceId, Tag, TagObservation},
    record::{Exclusions, Key},
};
use inventory_engine::inventory::{PageCommit, SchemaRegistry};
use serde_json::{Value as Json, json};

struct Source {
    service: &'static str,
    version: &'static str,
    parents: &'static [&'static str],
    operation: &'static str,
    input: &'static str,
    member: Option<&'static str>,
    map: bool,
}

fn decode(plan: &CallablePlan<'_>, body: &Json) -> TestResult<call::DecodedResponse> {
    let protocol = match plan.protocol() {
        RuntimeProtocol::AwsJson10 => aws_wire::request::Protocol::AwsJson10,
        RuntimeProtocol::AwsJson11 => aws_wire::request::Protocol::AwsJson11,
        RuntimeProtocol::RestJson1 => aws_wire::request::Protocol::RestJson1,
        _ => return Err("fixture requires JSON wire protocol".into()),
    };
    let body = serde_json::to_vec(body)?;
    let record = aws_wire::response::decode(aws_wire::response::ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol,
        body: &body,
        headers: &http::HeaderMap::new(),
        status: http::StatusCode::OK,
        limits: aws_wire::response::DecodeLimits::default().preserve_field_presence(),
    })?;
    Ok(call::DecodedResponse {
        record,
        headers: http::HeaderMap::new(),
    })
}

#[tokio::test]
async fn given_reviewed_list_and_map_sources_when_consumed_then_only_trusted_parent_keys_receive_tags()
-> TestResult {
    let sources = [
        Source {
            service: "acm",
            version: "2015-12-08",
            parents: &["ListCertificates"],
            operation: "ListTagsForCertificate",
            input: "CertificateArn",
            member: Some("CertificateArn"),
            map: false,
        },
        Source {
            service: "global-accelerator",
            version: "2018-08-08",
            parents: &["ListAccelerators", "ListCustomRoutingAccelerators"],
            operation: "ListTagsForResource",
            input: "ResourceArn",
            member: Some("AcceleratorArn"),
            map: false,
        },
        Source {
            service: "license-manager",
            version: "2018-08-01",
            parents: &["ListLicenseConfigurations"],
            operation: "ListTagsForResource",
            input: "ResourceArn",
            member: Some("LicenseConfigurationArn"),
            map: false,
        },
        Source {
            service: "medical-imaging",
            version: "2023-07-19",
            parents: &["ListDatastores"],
            operation: "ListTagsForResource",
            input: "resourceArn",
            member: None,
            map: true,
        },
    ];
    for source in sources {
        let operations = source
            .parents
            .iter()
            .copied()
            .chain(std::iter::once(source.operation))
            .collect::<Vec<_>>();
        let compiled = fixture::compile(source.service, source.version, &operations)?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let tag_plan = catalog
            .lookup(source.service, source.operation)
            .ok_or("tag endpoint missing")?;
        assert!(matches!(tag_plan.purpose(), PlanPurpose::TagEndpoint(_)));
        let directory = tempfile::tempdir()?;
        let mut store = BinaryStore::create(
            &directory.path().join("tags.sqlite"),
            SchemaRegistry::new(&compiled.bytes, catalog.schemas().to_vec())?,
        )
        .await?;
        for (index, operation) in source.parents.iter().enumerate() {
            let sweep = SweepId::new(format!("{}-{index}", source.service))?;
            let parent_plan = catalog
                .lookup(source.service, operation)
                .ok_or("reviewed parent missing")?;
            let owner_id = format!(
                "arn:aws:{}:us-east-1:123456789012:resource/parent-{index}",
                source.service
            );
            let key = ResourceKey {
                account: AccountId::parse("123456789012")?,
                region: Region::parse("us-east-1")?,
                kind: parent_plan.kind().clone(),
                id: ResourceId::parse(&owner_id)?,
            };
            let row = Record::encode(&Value::Map(
                source
                    .member
                    .map(|member| -> TestResult<_> {
                        Ok((
                            Key::Field(compiled.fields.id(member).ok_or("parent member missing")?),
                            Value::Text(owner_id.clone()),
                        ))
                    })
                    .transpose()?
                    .into_iter()
                    .collect(),
            ))?;
            let sentinel_id = format!("{owner_id}-sentinel");
            let sentinel = ResourceKey {
                id: ResourceId::parse(&sentinel_id)?,
                ..key.clone()
            };
            let sentinel_row = Record::encode(&Value::Map(
                source
                    .member
                    .map(|member| -> TestResult<_> {
                        Ok((
                            Key::Field(compiled.fields.id(member).ok_or("parent member missing")?),
                            Value::Text(sentinel_id.clone()),
                        ))
                    })
                    .transpose()?
                    .into_iter()
                    .collect(),
            ))?;
            let sentinel_observation =
                Observation::new(sentinel.clone(), sentinel_row, &Exclusions::none(), None)?;
            let observation =
                Observation::new(key.clone(), row.clone(), &Exclusions::none(), None)?;
            let request = input::root_bindings(parent_plan, Some(&key.account), Some(&key.region))?;
            let context = ChildContext {
                parent_plan,
                plan: tag_plan,
                request: &request,
                sweep: &sweep,
                discovery: &[],
                branch: None,
            };
            let mut budget =
                super::super::super::child::PageContextBudget::new(&DiscoveryContext::empty(), 0)?;
            let work = child_work(&context, &row, &observation, &mut budget)?
                .ok_or("reviewed source was not scheduled")?;
            if source.member.is_some() {
                assert!(
                    child_work(&context, &empty()?, &observation, &mut budget)?.is_none(),
                    "missing reviewed request source must not schedule an invented identifier"
                );
            }
            let scope = ScopeKey::new(
                key.account.clone(),
                key.region.clone(),
                ServiceId::parse(source.service)?,
                OperationId::parse(operation)?,
                None,
            )?;
            store
                .seed_work(WorkSpec::new(
                    sweep.clone(),
                    scope,
                    None,
                    request.as_bytes().to_vec(),
                )?)
                .await?;
            let parent_lease = claim(&store, &sweep, 0).await?;
            store
                .commit_page(PageCommit {
                    lease: parent_lease,
                    observations: vec![observation, sentinel_observation],
                    edges: vec![],
                    unresolved: vec![],
                    tags: vec![TagObservation::new(
                        sentinel.clone(),
                        TagCoverage::Complete,
                        vec![Tag::new("sentinel", "untouched")?],
                    )?],
                    next_token: None,
                    continuation_digest: None,
                    children: vec![work],
                    done: true,
                    observed_at_ms: 1,
                })
                .await?;
            let lease = claim(&store, &sweep, 2).await?;
            let bindings = Record::from_bytes(lease.bindings().to_vec())?;
            assert!(
                matches!(bindings.root().get_field(compiled.fields.id(source.input).ok_or("request member missing")?)?, Some(ValueRef::Text(value)) if value == owner_id)
            );
            let body = if source.map {
                json!({"tags":{"environment":"production","empty":""},"resourceArn":"attacker-owner"})
            } else {
                json!({"Tags":[{"Key":"environment","Value":"production"},{"Key":"empty","Value":""}],"ResourceArn":"attacker-owner"})
            };
            let parts = prepare_page(
                &catalog,
                &key.account,
                10,
                &lease,
                tag_plan,
                decode_cursor(lease.token())?,
                decode(tag_plan, &body)?,
            )?;
            assert!(parts.observations.is_empty());
            assert!(parts.next.is_none());
            let snapshot = pair(&parts.tags)?;
            assert_eq!(snapshot.resource(), &key);
            assert_eq!(snapshot.coverage(), TagCoverage::Complete);
            assert_eq!(
                snapshot
                    .tags()
                    .iter()
                    .map(|pair| (pair.key(), pair.value()))
                    .collect::<Vec<_>>(),
                vec![("empty", ""), ("environment", "production")]
            );
            if source.map {
                let absent = prepare_page(
                    &catalog,
                    &key.account,
                    10,
                    &lease,
                    tag_plan,
                    decode_cursor(lease.token())?,
                    decode(tag_plan, &json!({}))?,
                )?;
                assert_eq!(pair(&absent.tags)?.coverage(), TagCoverage::Unobserved);
                let explicit_empty = prepare_page(
                    &catalog,
                    &key.account,
                    10,
                    &lease,
                    tag_plan,
                    decode_cursor(lease.token())?,
                    decode(tag_plan, &json!({"tags":{}}))?,
                )?;
                assert_eq!(
                    pair(&explicit_empty.tags)?.coverage(),
                    TagCoverage::Complete
                );
                assert!(pair(&explicit_empty.tags)?.tags().is_empty());
                assert!(decode(tag_plan, &json!({"tags":{"known":null}})).is_err());
            } else {
                assert!(decode(tag_plan, &json!({"Tags":[null]})).is_err());
                assert!(decode(tag_plan, &json!({"Tags":[{"Key":"known","Value":null}]})).is_err());
            }
            commit(&store, lease, parts, 3).await?;
            assert_eq!(
                tags(&store, &key).await?,
                vec![
                    ("empty".into(), "".into()),
                    ("environment".into(), "production".into())
                ]
            );
            for (suffix, body, expected) in [
                (
                    "absent",
                    json!({}),
                    vec![
                        ("empty".to_owned(), "".to_owned()),
                        ("environment".to_owned(), "production".to_owned()),
                    ],
                ),
                (
                    "empty",
                    if source.map {
                        json!({"tags":{}})
                    } else {
                        json!({"Tags":[]})
                    },
                    Vec::new(),
                ),
            ] {
                let attempt = SweepId::new(format!("{}-{index}-{suffix}", source.service))?;
                let scope = ScopeKey::new(
                    key.account.clone(),
                    key.region.clone(),
                    ServiceId::parse(source.service)?,
                    OperationId::parse(source.operation)?,
                    Some(key.clone()),
                )?;
                store
                    .seed_work(WorkSpec::new(
                        attempt.clone(),
                        scope,
                        None,
                        bindings.as_bytes().to_vec(),
                    )?)
                    .await?;
                drop(store);
                store = BinaryStore::open(&directory.path().join("tags.sqlite")).await?;
                let lease = claim(&store, &attempt, 4).await?;
                let parts = prepare_page(
                    &catalog,
                    &key.account,
                    10,
                    &lease,
                    tag_plan,
                    decode_cursor(lease.token())?,
                    decode(tag_plan, &body)?,
                )?;
                commit(&store, lease, parts, 5).await?;
                drop(store);
                store = BinaryStore::open(&directory.path().join("tags.sqlite")).await?;
                assert_eq!(tags(&store, &key).await?, expected);
                assert_eq!(
                    tags(&store, &sentinel).await?,
                    vec![("sentinel".into(), "untouched".into())]
                );
            }
        }
        drop(store);
        directory.close()?;
    }
    Ok(())
}
