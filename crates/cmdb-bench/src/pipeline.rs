use super::{
    ensure_fresh, pipeline_fixture as fixture, pipeline_native as native, pipeline_oracle as oracle,
};
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{
    BinaryStore, ChangeStats, PageCommit, SchemaRegistry, SweepId, WorkLease, WorkSpec,
};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};
#[cfg(test)]
#[path = "pipeline_drain_tests.rs"]
mod drain_tests;

pub(super) type BenchResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;
const MAX_PAGES: usize = 100_000;
const MAX_ROWS: usize = 10_000_000;

#[derive(Clone, Debug)]
pub(super) struct Workload {
    pub path: PathBuf,
    pub catalog: PathBuf,
    pub pages: usize,
    pub rows: usize,
    pub concurrency: usize,
    pub graph: Option<native::Target>,
}

impl Workload {
    pub fn validate(&self) -> BenchResult {
        if self.pages == 0
            || self.pages > MAX_PAGES
            || self.rows > 1000
            || !(1..=64).contains(&self.concurrency)
        {
            return Err("pages must be 1..100000, rows/page 0..1000, concurrency 1..64".into());
        }
        let total = self.resources()?;
        if total > MAX_ROWS {
            return Err("workload exceeds 10000000 resources".into());
        }
        if let Some(target) = &self.graph {
            target.validate()?;
            if total == 0 {
                return Err("native graph scenario requires at least one resource".into());
            }
        }
        Ok(())
    }
    pub fn rows_per_scope(&self) -> BenchResult<usize> {
        self.pages
            .checked_mul(self.rows)
            .ok_or_else(|| "resource count overflow".into())
    }
    pub fn resources(&self) -> BenchResult<usize> {
        self.rows_per_scope()?
            .checked_mul(self.concurrency)
            .ok_or_else(|| "resource count overflow".into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Initial,
    TagUpdate,
    Replay,
}
impl Phase {
    pub fn name(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::TagUpdate => "tag_update",
            Self::Replay => "duplicate_replay",
        }
    }
    pub fn timestamp(self) -> i64 {
        match self {
            Self::Initial => 1000,
            Self::TagUpdate => 2000,
            Self::Replay => 3000,
        }
    }
}

#[derive(Default)]
struct Totals {
    pages: u64,
    inserted: u64,
    changed: u64,
    deleted: u64,
    decode: Duration,
    commit: Duration,
}
impl Totals {
    fn add(&mut self, stats: ChangeStats, decode: Duration, commit: Duration) -> BenchResult {
        self.pages = self.pages.checked_add(1).ok_or("page counter overflow")?;
        self.inserted = self
            .inserted
            .checked_add(stats.resources_inserted)
            .ok_or("insert counter overflow")?;
        self.changed = self
            .changed
            .checked_add(stats.resources_changed)
            .ok_or("change counter overflow")?;
        self.deleted = self
            .deleted
            .checked_add(stats.resources_deleted)
            .ok_or("delete counter overflow")?;
        self.decode = self
            .decode
            .checked_add(decode)
            .ok_or("decode duration overflow")?;
        self.commit = self
            .commit
            .checked_add(commit)
            .ok_or("commit duration overflow")?;
        Ok(())
    }
}

fn read_catalog(path: &std::path::Path) -> BenchResult<Vec<u8>> {
    let limit = inventory_core::catalog::MAX_CATALOG_BYTES;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(u64::try_from(limit)?.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("catalog exceeds the runtime catalog bound".into());
    }
    Ok(bytes)
}

pub(super) async fn run(workload: Workload) -> BenchResult {
    workload.validate()?;
    ensure_fresh(&workload.path).map_err(|error| error.to_string())?;
    let bytes = read_catalog(&workload.catalog)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    if catalog.lookup("dynamodb", "ListTables").is_none() {
        return Err("compiled catalog lacks reviewed dynamodb/ListTables; compile current embedded catalog first".into());
    }
    if let Some(target) = &workload.graph {
        native::require_fresh(target).await?;
    }
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let started = Instant::now();
    let store = BinaryStore::create(&workload.path, registry).await?;
    println!(
        "pipeline_setup_us={} resources={} pages_per_scope={} rows_per_page={} concurrency={} max_inflight_rows={} root_drain_work_measured=false allocation_profile=not_collected inflight_error_policy=drain_all durability=production_defaults database={}",
        started.elapsed().as_micros(),
        workload.resources()?,
        workload.pages,
        workload.rows,
        workload.concurrency,
        workload
            .rows
            .checked_mul(workload.concurrency)
            .ok_or("inflight bound overflow")?,
        workload.path.display()
    );
    stream::iter([Phase::Initial, Phase::TagUpdate, Phase::Replay])
        .then(|phase| run_phase(&workload, &catalog, &store, phase))
        .try_fold((), |(), ()| async { Ok(()) })
        .await
}

async fn run_phase(
    workload: &Workload,
    catalog: &RuntimeCatalog<'_>,
    store: &BinaryStore,
    phase: Phase,
) -> BenchResult {
    let sweep = SweepId::new(format!("cmdb-bench-{}", phase.name()))?;
    let works = (0..workload.concurrency)
        .map(|shard| {
            Ok(WorkSpec::new(
                sweep.clone(),
                fixture::scope(shard)?,
                None,
                Vec::new(),
            )?)
        })
        .collect::<BenchResult<Vec<_>>>()?;
    let revision_before = oracle::revision(&workload.path).await?;
    let started = Instant::now();
    store.seed_works(&works).await?;
    let totals = stream::iter(0..workload.pages)
        .then(|page| run_round(workload, catalog, store, &sweep, phase, page))
        .try_fold(Totals::default(), |mut total, round| async move {
            round
                .into_iter()
                .try_for_each(|(stats, decode, commit)| total.add(stats, decode, commit))?;
            Ok(total)
        })
        .await?;
    let elapsed = started.elapsed();
    let expected_inserted = if phase == Phase::Initial {
        u64::try_from(workload.resources()?)?
    } else {
        0
    };
    let expected_pages = u64::try_from(
        workload
            .pages
            .checked_mul(workload.concurrency)
            .ok_or("page count overflow")?,
    )?;
    if totals.inserted != expected_inserted
        || totals.changed != 0
        || totals.deleted != 0
        || totals.pages != expected_pages
    {
        return Err(format!(
            "{} commit statistics mismatch: pages={} inserted={} changed={} deleted={}",
            phase.name(),
            totals.pages,
            totals.inserted,
            totals.changed,
            totals.deleted
        )
        .into());
    }
    oracle::check(workload, store, &sweep, phase, revision_before).await?;
    println!(
        "pipeline_phase={} pages={} resources={} inserted={} changed={} deleted={} wall_us={} pages_per_second={:.3} resources_per_second={:.3} decode_worker_us={} commit_worker_us={} timings_include=fixture_generation_decode_tag_normalization_claim_commit timings_exclude=setup_state_oracles_native_projection",
        phase.name(),
        totals.pages,
        workload.resources()?,
        totals.inserted,
        totals.changed,
        totals.deleted,
        elapsed.as_micros(),
        rate(expected_pages, elapsed)?,
        rate(u64::try_from(workload.resources()?)?, elapsed)?,
        totals.decode.as_micros(),
        totals.commit.as_micros()
    );
    if let Some(target) = &workload.graph {
        native::consume_and_check(workload, store, target, phase).await?;
    }
    Ok(())
}

pub(super) fn rate(count: u64, duration: Duration) -> BenchResult<f64> {
    if duration.is_zero() {
        return Err("measurement clock returned zero elapsed time".into());
    }
    // Every validated workload is below u32::MAX, so presentation is lossless.
    Ok(f64::from(u32::try_from(count)?) / duration.as_secs_f64())
}

type PageResult = (ChangeStats, Duration, Duration);
async fn run_round(
    workload: &Workload,
    catalog: &RuntimeCatalog<'_>,
    store: &BinaryStore,
    sweep: &SweepId,
    phase: Phase,
    page: usize,
) -> BenchResult<Vec<PageResult>> {
    // Claim the bounded batch first: no scope can be scheduled twice in a round.
    let leases = stream::iter(0..workload.concurrency)
        .then(|_| async {
            let lease: BenchResult<WorkLease> = store
                .claim(sweep, phase.timestamp(), 60_000)
                .await?
                .ok_or_else(|| "benchmark work lease is missing".into());
            lease
        })
        .try_collect::<Vec<WorkLease>>()
        .await?;
    let results = stream::iter(leases)
        .map(|lease| commit(workload, catalog, store, lease, phase, page))
        .buffer_unordered(workload.concurrency)
        .collect::<Vec<_>>()
        .await;
    // Drain every in-flight commit before returning any failure.
    results.into_iter().collect()
}

async fn commit(
    workload: &Workload,
    catalog: &RuntimeCatalog<'_>,
    store: &BinaryStore,
    lease: WorkLease,
    phase: Phase,
    page: usize,
) -> BenchResult<PageResult> {
    let plan = catalog
        .lookup("dynamodb", "ListTables")
        .ok_or("ListTables plan disappeared")?;
    let fixture = fixture::TablePage {
        start: page
            .checked_mul(workload.rows)
            .ok_or("page offset overflow")?,
        rows: workload.rows,
    };
    let started = Instant::now();
    let observations = fixture::decode(plan, lease.scope(), &fixture)?;
    let tags = fixture::tags(&observations, fixture.start, phase)?;
    let decode = started.elapsed();
    let next_page = page.checked_add(1).ok_or("page index overflow")?;
    let done = next_page == workload.pages;
    let next_token = if done {
        None
    } else {
        Some(u64::try_from(next_page)?.to_be_bytes().to_vec())
    };
    let continuation_digest = next_token
        .as_ref()
        .map(|token| <[u8; 32]>::from(Sha256::digest(token)));
    let started = Instant::now();
    let result = store
        .commit_page(PageCommit {
            lease,
            observations,
            edges: Vec::new(),
            unresolved: Vec::new(),
            tags,
            next_token,
            continuation_digest,
            children: Vec::new(),
            done,
            observed_at_ms: phase.timestamp(),
        })
        .await?;
    if result.repeated_token {
        return Err("deterministic distinct page token was repeated".into());
    }
    Ok((result.stats, decode, started.elapsed()))
}
