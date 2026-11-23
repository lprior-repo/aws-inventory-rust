use super::{TestResult, compile, cursor::Cursor, decode};
use crate::{
    execution::page::{child, rows},
    input, record, request,
};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{
        AccountId, DiscoveryContext, OperationId, Region, RegionId, ResourceKey, ScopeKey,
        ServiceId,
    },
    record::{Record, ValueRef},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn rule(name: &str) -> Value {
    json!({"ruleName":name,"filter":{"isPublic":{"eq":["true"]}},
        "createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z"})
}

fn text(
    record: &Record,
    fields: &catalog_tool::smithy::Registry,
    field: &str,
) -> TestResult<String> {
    match record::path_value(
        record.root(),
        &[fields.id(field).ok_or("modeled field absent")?],
    )? {
        Some(ValueRef::Text(value)) => Ok(value.to_owned()),
        _ => Err(format!("required text absent: {field}").into()),
    }
}

#[tokio::test]
async fn given_two_analyzers_with_identical_rule_names_when_later_pages_and_details_resume_then_keys_and_edges_keep_their_analyzer()
-> TestResult {
    // Given actual pinned shapes and reviewed policies, including both list and detail facets.
    let compiled = compile(
        "accessanalyzer",
        "2019-11-01",
        &["ListAnalyzers", "ListArchiveRules", "GetArchiveRule"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let consumers = crate::execution::page::consumer_index::ConsumerIndex::build(&catalog)?;
    let parent = catalog
        .lookup("accessanalyzer", "ListAnalyzers")
        .ok_or("analyzer list absent")?;
    let list = catalog
        .lookup("accessanalyzer", "ListArchiveRules")
        .ok_or("archive list absent")?;
    let detail = catalog
        .lookup("accessanalyzer", "GetArchiveRule")
        .ok_or("archive detail absent")?;
    assert_eq!(list.kind().as_str(), "AWS::AccessAnalyzer::ArchiveRule");
    assert_eq!(list.kind(), detail.kind());
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let region = Region::Regional(home.clone());
    let sweep = SweepId::new("archive-rule-correlation".to_owned())?;
    let scope = ScopeKey::new(
        account.clone(),
        region.clone(),
        ServiceId::parse("accessanalyzer")?,
        OperationId::parse("ListAnalyzers")?,
        None,
    )?;
    let root_request = input::root_bindings(parent, Some(&account), Some(&region))?;
    let response = decode(
        parent,
        &json!({"analyzers":[
            {"arn":"arn:aws:access-analyzer:us-east-1:123456789012:analyzer/analyzer-a","name":"analyzer-a","type":"ACCOUNT","status":"ACTIVE","createdAt":"2026-01-01T00:00:00Z"},
            {"arn":"arn:aws:access-analyzer:us-east-1:123456789012:analyzer/analyzer-b","name":"analyzer-b","type":"ACCOUNT","status":"ACTIVE","createdAt":"2026-01-01T00:00:00Z"}
        ]}),
    )?;
    let context = child::Context::new(parent, &sweep, &account, &home, &consumers);
    let (analyzers, children) = child::policy_dfgk_tests::project_children(
        &catalog,
        &context,
        &scope,
        &root_request,
        &response,
        &DiscoveryContext::empty(),
    )?;
    assert_eq!(analyzers.len(), 2);
    assert_eq!(children.len(), 2);
    for work in &children {
        assert_eq!(work.scope().operation().as_str(), "ListArchiveRules");
        assert!(
            analyzers
                .iter()
                .any(|analyzer| Some(analyzer.key()) == work.scope().parent())
        );
    }
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("archive-rules.sqlite");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let mut store = BinaryStore::create(&path, registry).await?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            root_request.as_bytes().to_vec(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, 0, 10_000)
        .await?
        .ok_or("root lease absent")?;
    store
        .commit_page(PageCommit {
            lease,
            observations: analyzers,
            children,
            unresolved: Vec::new(),
            edges: Vec::new(),
            tags: Vec::new(),
            next_token: None,
            continuation_digest: None,
            done: true,
            observed_at_ms: 0,
        })
        .await?;
    let mut keys = BTreeMap::<(String, String), ResourceKey>::new();
    let mut resumed_analyzers = Vec::new();
    let mut detail_count = 0_u32;
    // When the real queue, context projection, cursor and request serializers run after each reopen.
    for at in 1..=16 {
        let Some(lease) = store.claim(&sweep, at, 10_000).await? else {
            break;
        };
        let plan = catalog
            .lookup(
                lease.scope().service().as_str(),
                lease.scope().operation().as_str(),
            )
            .ok_or("leased plan absent")?;
        let base = Record::from_bytes(lease.bindings().to_vec())?;
        let ancestry = DiscoveryContext::decode(lease.context())?;
        let analyzer = text(&base, &compiled.fields, "analyzerName")?;
        assert!(matches!(analyzer.as_str(), "analyzer-a" | "analyzer-b"));
        let cursor = Cursor::decode(lease.token())?;
        let fields = cursor.request_fields(plan.pagination())?;
        let actual_request = input::merge_input(&base, &fields)?;
        let wire = request::encode(plan, "https://fixture.example.test", &base, &fields)?;
        assert!(
            wire.uri()
                .path()
                .contains(&format!("/analyzer/{analyzer}/archive-rule"))
        );
        let (name, response_body) = match plan.operation_name() {
            "ListArchiveRules" if lease.token().is_none() => {
                assert!(
                    lease
                        .scope()
                        .parent()
                        .ok_or("archive work lost analyzer parent")?
                        .id
                        .as_str()
                        .ends_with(&format!("/{analyzer}"))
                );
                (
                    "early".to_owned(),
                    json!({"archiveRules":[rule("early")],"nextToken":"later-rules"}),
                )
            }
            "ListArchiveRules" => {
                assert_eq!(
                    text(&actual_request, &compiled.fields, "nextToken")?,
                    "later-rules"
                );
                assert!(
                    wire.uri()
                        .query()
                        .ok_or("later-page query absent")?
                        .contains("nextToken=later-rules")
                );
                resumed_analyzers.push(analyzer.clone());
                (
                    "shared".to_owned(),
                    json!({"archiveRules":[rule("shared")]}),
                )
            }
            "GetArchiveRule" => {
                detail_count = detail_count.saturating_add(1);
                let name = text(&base, &compiled.fields, "ruleName")?;
                (name.clone(), json!({"archiveRule":rule(&name)}))
            }
            _ => return Err("unexpected archive child operation".into()),
        };
        let response = decode(plan, &response_body)?;
        let context = child::Context::new(plan, &sweep, &account, &home, &consumers);
        let (observations, children) = child::policy_dfgk_tests::project_children(
            &catalog,
            &context,
            lease.scope(),
            &actual_request,
            &response,
            &ancestry,
        )?;
        assert_eq!(observations.len(), 1);
        let selected = record::item_records(&response, plan.items())?;
        let row = selected.first().ok_or("archive rule row absent")?;
        assert!(
            record::path_value(
                row.root(),
                &[compiled
                    .fields
                    .id("analyzerName")
                    .ok_or("analyzer request field absent")?]
            )?
            .is_none(),
            "parent fields must not be manufactured in selected rows"
        );
        assert!(
            record::resource_id_with_context(row, plan.identity(), None, &ancestry, &root_request)
                .is_err(),
            "ruleName-only fallback must fail"
        );
        let observation = observations.first().ok_or("archive observation absent")?;
        assert_eq!(
            observation.key().id.as_str(),
            format!("{}:{analyzer}|{}:{name}", analyzer.len(), name.len())
        );
        if let Some(previous) =
            keys.insert((analyzer.clone(), name.clone()), observation.key().clone())
        {
            assert_eq!(
                &previous,
                observation.key(),
                "list/detail identities diverged"
            );
        }
        let unresolved = rows::references(
            &catalog,
            plan,
            &home,
            &selected,
            &observations,
            &actual_request,
            None,
        )?;
        assert_eq!(unresolved.len(), 1);
        let reference = unresolved.first().ok_or("analyzer reference absent")?;
        assert_eq!(&reference.from, observation.key());
        assert_eq!(reference.relation.as_str(), "IN_ANALYZER");
        assert_eq!(
            reference.target.kind().as_str(),
            "AWS::AccessAnalyzer::Analyzer"
        );
        assert_eq!(reference.target.value().as_str(), analyzer);
        assert_eq!(reference.target.account(), &account);
        assert_eq!(reference.target.region(), &region);
        let next = cursor.next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?;
        let done = next.is_none();
        let (next_token, continuation_digest) = next.map_or((None, None), |next| {
            (Some(next.cursor.into_bytes()), next.continuation_digest)
        });
        store
            .commit_page(PageCommit {
                lease,
                observations,
                children,
                unresolved,
                edges: Vec::new(),
                tags: Vec::new(),
                next_token,
                continuation_digest,
                done,
                observed_at_ms: at,
            })
            .await?;
        drop(store);
        store = BinaryStore::open(&path).await?;
    }
    // Then equal names have two stable owners, both later pages and all detail facets ran, and graph joins remain exact.
    assert!(store.claim(&sweep, 17, 10_000).await?.is_none());
    resumed_analyzers.sort();
    assert_eq!(resumed_analyzers, ["analyzer-a", "analyzer-b"]);
    assert_eq!(detail_count, 4);
    assert_eq!(keys.len(), 4);
    assert_ne!(
        keys.get(&("analyzer-a".to_owned(), "shared".to_owned())),
        keys.get(&("analyzer-b".to_owned(), "shared".to_owned()))
    );
    let resources = store.projection_resources(None, 100).await?;
    assert_eq!(
        resources
            .iter()
            .filter(|resource| resource.key.kind == *list.kind())
            .count(),
        4
    );
    let edges = store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 4);
    for edge in edges.edges {
        assert_eq!(edge.relation.as_str(), "IN_ANALYZER");
        let analyzer = edge
            .to
            .id
            .as_str()
            .rsplit('/')
            .next()
            .ok_or("analyzer ARN suffix absent")?;
        assert!(
            edge.from
                .id
                .as_str()
                .starts_with(&format!("{}:{analyzer}|", analyzer.len()))
        );
    }
    Ok(())
}
