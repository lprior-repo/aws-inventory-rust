use super::*;
mod budget;
use crate::{execution::now_ms, record::policy_dfgk_tests::fixture};
use inventory_core::{
    inventory::{OperationId, Region, ScopeKey, ServiceId},
    record::{Record, Value},
};
use inventory_engine::{
    inventory::{BinaryStore, SchemaRegistry, SweepId, WorkSpec},
    limiter::{ApiError, ApiExecutor, ExecutorConfig},
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Scenario {
    directory: tempfile::TempDir,
    store: BinaryStore,
    catalog: RuntimeCatalog<'static>,
    sweep: SweepId,
    account: AccountId,
    home: RegionId,
}

impl Scenario {
    async fn new(name: &str) -> TestResult<Self> {
        let catalog = fixture::catalog()?;
        let directory = tempfile::tempdir()?;
        let store = BinaryStore::create(
            &directory.path().join("assembly.sqlite"),
            SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?,
        )
        .await?;
        let sweep = SweepId::new(name.to_owned())?;
        let account = AccountId::parse("123456789012")?;
        let home = RegionId::parse("us-east-1")?;
        let scope = ScopeKey::new(
            account.clone(),
            Region::Regional(home.clone()),
            ServiceId::parse("iottwinmaker")?,
            OperationId::parse("ListWorkspaces")?,
            None,
        )?;
        store
            .seed_work(WorkSpec::new(sweep.clone(), scope, None, Vec::new())?)
            .await?;
        Ok(Self {
            directory,
            store,
            catalog,
            sweep,
            account,
            home,
        })
    }

    async fn lease(&self) -> TestResult<WorkLease> {
        Ok(self
            .store
            .claim(&self.sweep, now_ms()?, 3_600_000)
            .await?
            .ok_or("assembly lease absent")?)
    }

    fn response(&self, count: usize) -> TestResult<call::DecodedResponse> {
        let plan = fixture::plan(&self.catalog, "iottwinmaker", "ListWorkspaces")?;
        let rows = (0..count).map(|ordinal| serde_json::json!({
            "workspaceId":format!("workspace-{ordinal}"),
            "arn":format!("arn:aws:iottwinmaker:us-east-1:123456789012:workspace/workspace-{ordinal}"),
            "creationDateTime":0,"updateDateTime":0,
        })).collect::<Vec<_>>();
        let bytes = serde_json::to_vec(&serde_json::json!({"workspaceSummaries":rows}))?;
        Ok(call::DecodedResponse {
            record: fixture::decode(plan, &bytes)?,
            headers: http::HeaderMap::new(),
        })
    }

    fn request(&self) -> TestResult<PageRequest> {
        Ok(PageRequest {
            record: Record::encode(&Value::Map(BTreeMap::new()))?,
            region: self.home.clone(),
        })
    }

    async fn assert_atomic(&self) -> TestResult {
        let before = self.store.sweep_summary(&self.sweep).await?;
        assert_eq!(before.resources_observed, 0);
        assert_eq!(before.work_pending, 1);
        assert_eq!(before.complete_scopes, 0);
        tokio::task::yield_now().await;
        assert_eq!(self.store.sweep_summary(&self.sweep).await?, before);
        assert!(self.directory.path().join("assembly.sqlite").exists());
        Ok(())
    }
}

#[tokio::test]
async fn given_large_real_workspace_page_when_projected_then_heartbeat_runs_between_bounded_chunks_and_ordinals_remain_absolute()
-> TestResult {
    let scenario = Scenario::new("large-page-heartbeat").await?;
    let lease = scenario.lease().await?;
    let plan = fixture::plan(&scenario.catalog, "iottwinmaker", "ListWorkspaces")?;
    let consumers = ConsumerIndex::build(&scenario.catalog)?;
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let context = Context {
        catalog: &scenario.catalog,
        account: &scenario.account,
        home: &scenario.home,
        executor: &executor,
        consumers: &consumers,
        max_pages: 10,
    };
    let response = scenario.response(4096)?;
    let request = scenario.request()?;
    let ancestry = DiscoveryContext::empty();
    let selected = record::item_records(&response.record, plan.items())?;
    let boundary_row = resource_projection::project(
        plan,
        selected
            .get(ROWS_PER_CHECKPOINT)
            .ok_or("chunk-boundary row absent")?,
    )?;
    let expected_boundary = ancestry
        .push(inventory_core::inventory::DiscoveryFrame::project(
            lease.scope().service().clone(),
            lease.scope().operation().clone(),
            u32::try_from(ROWS_PER_CHECKPOINT)?,
            plan.context_projection(),
            &boundary_row,
            &response.record,
            &request.record,
        )?)?
        .discriminator()?;
    drop(selected);
    let done = AtomicBool::new(false);
    let beats = AtomicUsize::new(0);
    let assembly = async {
        let result = prepare(
            &context,
            &lease,
            plan,
            cursor::Cursor::decode(None)?,
            response,
            &request,
            &ancestry,
        )
        .await;
        done.store(true, Ordering::SeqCst);
        result
    };
    let heartbeat = async {
        while !done.load(Ordering::SeqCst) {
            beats.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
        }
    };
    let (parts, ()) = tokio::join!(assembly, heartbeat);
    let parts = parts?;
    assert!(
        beats.load(Ordering::SeqCst) >= 4096 / 32,
        "large page exceeded the declared 32-row cooperative chunk budget"
    );
    assert_eq!(parts.observations.len(), 4096);
    assert_eq!(parts.children.len(), 4096);
    let unique = parts
        .children
        .iter()
        .map(|work| work.scope().context().map(|hash| *hash.as_bytes()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        unique.len(),
        4096,
        "chunk-local ordinals collided across one page"
    );
    assert_eq!(
        parts
            .children
            .get(ROWS_PER_CHECKPOINT)
            .ok_or("chunk-boundary child absent")?
            .scope()
            .context(),
        Some(expected_boundary),
        "ordinal restarted at the next chunk"
    );
    assert!(parts.next.is_none());
    scenario.assert_atomic().await?;
    Ok(())
}

#[tokio::test]
async fn given_large_page_when_heartbeat_requests_cancellation_then_projection_stops_without_partial_resource_or_cursor_commit()
-> TestResult {
    let scenario = Scenario::new("large-page-cancel").await?;
    let lease = scenario.lease().await?;
    let plan = fixture::plan(&scenario.catalog, "iottwinmaker", "ListWorkspaces")?;
    let consumers = ConsumerIndex::build(&scenario.catalog)?;
    let executor = ApiExecutor::new(ExecutorConfig::default())?;
    let context = Context {
        catalog: &scenario.catalog,
        account: &scenario.account,
        home: &scenario.home,
        executor: &executor,
        consumers: &consumers,
        max_pages: 10,
    };
    let response = scenario.response(4096)?;
    let request = scenario.request()?;
    let ancestry = DiscoveryContext::empty();
    let cancel = async {
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }
        executor.cancel();
    };
    let (result, ()) = tokio::join!(
        prepare(
            &context,
            &lease,
            plan,
            cursor::Cursor::decode(None)?,
            response,
            &request,
            &ancestry
        ),
        cancel
    );
    assert!(matches!(result, Err(SweepError::Api(ApiError::Cancelled))));
    scenario.assert_atomic().await?;
    Ok(())
}

