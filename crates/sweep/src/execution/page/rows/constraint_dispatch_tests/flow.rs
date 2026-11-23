use super::super::super::{
    PageParts, PageRequest, assembly, consumer_index::ConsumerIndex, cursor, preparation,
};
use super::{cbor, *};
use crate::{
    call,
    execution::{draining, now_ms, routing},
    input, request,
};
use inventory_engine::inventory::WorkLease;

fn protocol(plan: &CallablePlan<'_>) -> TestResult<aws_wire::request::Protocol> {
    use aws_wire::request::Protocol;
    use inventory_core::catalog::runtime::RuntimeProtocol;
    match plan.protocol() {
        RuntimeProtocol::RpcV2Cbor => Ok(Protocol::RpcV2Cbor),
        RuntimeProtocol::AwsJson11 => Ok(Protocol::AwsJson11),
        _ => Err("unexpected service-catalog fixture protocol".into()),
    }
}

pub(super) fn decode(
    plan: &CallablePlan<'_>,
    body: serde_json::Value,
) -> TestResult<call::DecodedResponse> {
    let protocol = protocol(plan)?;
    let bytes = match protocol {
        aws_wire::request::Protocol::RpcV2Cbor => cbor::from_json(&body)?,
        _ => serde_json::to_vec(&body)?,
    };
    Ok(call::DecodedResponse {
        http_version: http::Version::HTTP_11,
        record: aws_wire::response::decode(aws_wire::response::ResponseContext {
            service: plan.service(),
            operation: plan.operation(),
            protocol,
            body: &bytes,
            headers: &http::HeaderMap::new(),
            status: http::StatusCode::OK,
            limits: aws_wire::response::DecodeLimits::default(),
        })?,
        headers: http::HeaderMap::new(),
    })
}

fn wire_request(
    plan: &CallablePlan<'_>,
    lease: &WorkLease,
) -> TestResult<(PageRequest, serde_json::Value)> {
    let home = RegionId::parse("us-east-1")?;
    let region =
        routing::request_region(plan, lease.scope().region(), &home, lease.scope().branch())?;
    let endpoint = routing::resolve(plan, &region)?;
    let base = Record::from_bytes(lease.bindings().to_vec())?;
    let mut fields = cursor::Cursor::decode(lease.token())?.request_fields(plan.pagination())?;
    if let Some(size) = plan.pagination().page_size.as_ref() {
        fields.push(input::InputField {
            path: size.path.clone(),
            value: size.value(),
        });
    }
    let native = request::encode(plan, &endpoint.url, &base, &fields)?;
    let wire = match protocol(plan)? {
        aws_wire::request::Protocol::RpcV2Cbor if native.body().is_empty() => serde_json::json!({}),
        aws_wire::request::Protocol::RpcV2Cbor => cbor::to_json(native.body())?,
        _ => serde_json::from_slice(native.body())?,
    };
    Ok((
        PageRequest {
            record: input::merge_input(&base, &fields)?,
            region,
        },
        wire,
    ))
}

pub(super) async fn prepare(
    catalog: &RuntimeCatalog<'_>,
    lease: &WorkLease,
    body: serde_json::Value,
) -> TestResult<PageParts> {
    let plan = catalog
        .lookup(
            lease.scope().service().as_str(),
            lease.scope().operation().as_str(),
        )
        .ok_or("claimed production plan absent")?;
    let (request, wire) = wire_request(plan, lease)?;
    let ancestry = preparation::ancestry(lease.context(), lease.scope().context())?;
    if plan.operation_name() == OPERATION {
        assert_eq!(
            wire.get("PortfolioId").and_then(serde_json::Value::as_str),
            Some(PORTFOLIO)
        );
        assert!(
            wire.get("Type").is_none(),
            "one native call must contain all constraint types"
        );
        assert!(
            !lease.context().is_empty(),
            "constraint lease lost root discovery ancestry"
        );
        assert_eq!(Some(ancestry.discriminator()?), lease.scope().context());
        assert!(
            lease.scope().parent().is_some(),
            "constraint lease lost portfolio parent"
        );
    } else {
        assert_eq!(plan.operation_name(), "ListPortfolios");
        assert!(
            wire.get("PortfolioId").is_none(),
            "root request inherited synthetic child bindings"
        );
        assert!(lease.scope().parent().is_none());
    }
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let consumers = ConsumerIndex::build(catalog)?;
    let executor = inventory_engine::limiter::ApiExecutor::new(Default::default())?;
    let context = assembly::Context {
        catalog,
        account: &account,
        home: &home,
        executor: &executor,
        consumers: &consumers,
        max_pages: 10,
    };
    Ok(assembly::prepare(
        &context,
        lease,
        plan,
        cursor::Cursor::decode(lease.token())?,
        decode(plan, body)?,
        &request,
        &ancestry,
    )
    .await?)
}

pub(super) async fn commit(store: &BinaryStore, lease: WorkLease, parts: PageParts) -> TestResult {
    let done = parts.next.is_none();
    let continuation_digest = parts
        .next
        .as_ref()
        .and_then(|advance| advance.continuation_digest);
    store
        .commit_page(PageCommit {
            lease,
            observations: parts.observations,
            unresolved: parts.references,
            edges: Vec::new(),
            tags: parts.tags,
            children: parts.children,
            next_token: parts.next.map(|advance| advance.cursor.into_bytes()),
            continuation_digest,
            done,
            observed_at_ms: now_ms()?,
        })
        .await?;
    Ok(())
}

pub(super) async fn reopen(
    store: BinaryStore,
    root: &tempfile::TempDir,
    catalog: &RuntimeCatalog<'_>,
) -> TestResult<BinaryStore> {
    drop(store);
    Ok(
        BinaryStore::open_with_catalog(&root.path().join("inventory.db"), catalog.raw_bytes())
            .await?,
    )
}

pub(super) async fn rooted(
    catalog: &RuntimeCatalog<'_>,
) -> TestResult<(tempfile::TempDir, BinaryStore, WorkLease)> {
    let root = tempfile::tempdir()?;
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&root.path().join("inventory.db"), registry).await?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let sweep = SweepId::new("native-constraint-discovery".to_owned())?;
    let mut work = Vec::new();
    for plan in catalog.plans() {
        draining::append_plan_work(
            &account,
            &home,
            &sweep,
            plan,
            &[Region::Regional(home.clone())],
            &mut work,
        )?;
    }
    assert_eq!(
        work.len(),
        1,
        "constraint child must not be seeded as a parentless root"
    );
    assert_eq!(
        work.first()
            .ok_or("root absent")?
            .scope()
            .operation()
            .as_str(),
        "ListPortfolios"
    );
    store.seed_works(&work).await?;
    let lease = store
        .claim(&sweep, now_ms()?, 3_600_000)
        .await?
        .ok_or("root lease absent")?;
    let parts = prepare(
        catalog,
        &lease,
        json!({"PortfolioDetails":[{"Id":PORTFOLIO,"ARN":PORTFOLIO_ARN}]}),
    )
    .await?;
    assert_eq!(parts.observations.len(), 1);
    assert_eq!(
        parts
            .observations
            .first()
            .ok_or("portfolio absent")?
            .key()
            .id
            .as_str(),
        PORTFOLIO_ARN
    );
    assert_eq!(
        parts.children.len(),
        1,
        "root must discover exactly one native constraint operation"
    );
    assert_eq!(
        parts
            .children
            .first()
            .ok_or("child absent")?
            .scope()
            .operation()
            .as_str(),
        OPERATION
    );
    commit(&store, lease, parts).await?;
    let store = reopen(store, &root, catalog).await?;
    let lease = store
        .claim(&sweep, now_ms()?, 3_600_000)
        .await?
        .ok_or("discovered constraint lease absent")?;
    assert_eq!(lease.scope().operation().as_str(), OPERATION);
    assert_eq!(
        lease
            .scope()
            .parent()
            .ok_or("discovered portfolio parent absent")?
            .id
            .as_str(),
        PORTFOLIO_ARN
    );
    Ok((root, store, lease))
}

pub(super) async fn reader(root: &tempfile::TempDir) -> TestResult<sqlx::SqlitePool> {
    Ok(sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(root.path().join("inventory.db"))
                .read_only(true),
        )
        .await?)
}
