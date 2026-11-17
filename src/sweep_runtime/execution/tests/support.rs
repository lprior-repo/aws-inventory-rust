use super::*;
use inventory_core::inventory::ScopeOutcome;
use inventory_engine::inventory::{PageCommit, SchemaRegistry, WorkLease};

pub(super) type TestResult<T = ()> = Result<T,Box<dyn std::error::Error>>;

pub(super) struct Fixture {
    pub directory: tempfile::TempDir,
    pub store: BinaryStore,
    pub sweep: SweepId,
    pub scope: ScopeKey,
    pub now: i64,
    pub lease_ms: i64,
}

impl Fixture {
    pub async fn new(name: &str) -> TestResult<Self> {
        let catalog = crate::sweep_runtime::record::policy_dfgk_tests::fixture::catalog()?;
        let plan = catalog.lookup("iottwinmaker","ListWorkspaces").ok_or("producer absent")?;
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(),catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("scheduler.sqlite"),registry).await?;
        let sweep = SweepId::new(name.to_owned())?;
        let scope = ScopeKey::new(AccountId::parse("123456789012")?,Region::Regional(RegionId::parse("us-east-1")?),
            ServiceId::parse(plan.service_name())?,OperationId::parse(plan.operation_name())?,None)?;
        let now = now_ms()?;
        let lease_ms = lease_budget(executor_config(4)?)?;
        Ok(Self { directory,store,sweep,scope,now,lease_ms })
    }

    pub async fn seed(&self) -> TestResult<WorkLease> {
        self.store.seed_work(WorkSpec::new(self.sweep.clone(),self.scope.clone(),None,Vec::new())?).await?;
        Ok(self.store.claim(&self.sweep,self.now,self.lease_ms).await?.ok_or("work absent")?)
    }

    pub fn scope_for(&self, ordinal: u32) -> TestResult<ScopeKey> {
        let account = AccountId::parse(&format!("{:012}", 100_000_000_000_u64 + u64::from(ordinal)))?;
        Ok(ScopeKey::new(account, self.scope.region().clone(), self.scope.service().clone(), self.scope.operation().clone(), None)?)
    }

    pub async fn outcome(&self, expected: ScopeOutcome) -> TestResult {
        let summary = self.store.sweep_summary(&self.sweep).await?;
        assert_eq!(summary.work_pending,0);
        match expected {
            ScopeOutcome::Complete => assert_eq!(summary.complete_scopes,1),
            ScopeOutcome::Partial => assert_eq!(summary.partial_scopes,1),
            _ => return Err("unexpected scheduler fixture outcome".into()),
        }
        assert!(self.store.claim(&self.sweep,now_ms()?,self.lease_ms).await?.is_none());
        assert!(self.directory.path().join("scheduler.sqlite").exists());
        Ok(())
    }
}

pub(super) fn page(lease: WorkLease, at: i64, next: Option<Vec<u8>>) -> PageCommit {
    let done = next.is_none();
    PageCommit { lease,observations:Vec::new(),edges:Vec::new(),unresolved:Vec::new(),tags:Vec::new(),
        continuation_digest:next.as_ref().map(|_| [7;32]),next_token:next,children:Vec::new(),done,observed_at_ms:at }
}
