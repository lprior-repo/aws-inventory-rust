//! Reproducible synthetic stage measurements; never AWS throughput claims.
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_core::{CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source};
use inventory_engine::{
    limiter::{ApiError, ApiExecutor, ExecutorConfig},
    store::Store,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;
use std::time::{Duration, Instant};

type BenchResult<T> = Result<T, Box<dyn Error>>;
const NETWORK_OPERATIONS: u32 = 512;

fn main() -> BenchResult<()> {
    let rows = std::env::args()
        .nth(1)
        .map(|text| text.parse::<u32>())
        .transpose()?
        .unwrap_or(10_000);
    if !(1..=100_000).contains(&rows) {
        return Err("rows must be in 1..=100000".into());
    }
    // Keep durable SQLite measurements off the container's tmpfs /tmp. File
    // creation and cleanup occur outside the asynchronous executor.
    std::fs::create_dir_all("artifacts")?;
    let directory = tempfile::Builder::new()
        .prefix("inventory-benchmark-")
        .tempdir_in("artifacts")?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?;
    let result = runtime.block_on(measure(directory.path(), rows))?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[tracing::instrument(skip(path))]
async fn measure(path: &Path, rows: u32) -> BenchResult<Value> {
    let storage = storage_cases(path, rows).await?;
    let network = network_cases().await?;
    Ok(
        json!({"synthetic":true,"rows_per_storage_case":rows,"storage":storage,"scheduler":network,
        "runtime_worker_threads":4,"storage_location":"artifacts/ (workspace filesystem)",
        "journal_mode":"WAL","synchronous":"FULL","edges_per_resource":1,
        "interpretation":"local stage measurements; mock API latency; no live AWS benchmark"}),
    )
}

#[tracing::instrument(skip(path))]
async fn storage_cases(path: &Path, rows: u32) -> BenchResult<Vec<Value>> {
    stream::iter([1_u32, 100_u32])
        .map(Ok::<_, Box<dyn Error>>)
        .and_then(|page_size| storage_case(path, rows, page_size))
        .try_collect()
        .await
}

#[tracing::instrument(skip(path))]
async fn storage_case(path: &Path, rows: u32, page_size: u32) -> BenchResult<Value> {
    let store = Store::open(path.join(format!("batch-{page_size}.sqlite"))).await?;
    let result = ingest_and_measure(&store, rows, page_size).await;
    store.close().await;
    result
}

#[tracing::instrument(skip(store))]
async fn ingest_and_measure(store: &Store, rows: u32, page_size: u32) -> BenchResult<Value> {
    store.begin_scan("benchmark", 1).await?;
    let started = Instant::now();
    let count = rows.div_ceil(page_size);
    stream::iter(0..count)
        .map(Ok::<_, Box<dyn Error>>)
        .try_for_each(|page| async move {
            store
                .apply_page("benchmark", fixture(page, rows, page_size))
                .await?;
            Ok(())
        })
        .await?;
    let summary = store
        .finish_scan("benchmark", ScanStatus::Complete, 2)
        .await?;
    if summary.resources != u64::from(rows) || summary.edges != u64::from(rows) {
        return Err("benchmark persistence count mismatch".into());
    }
    let elapsed = started.elapsed().as_secs_f64();
    Ok(
        json!({"resources_per_transaction":page_size,"resources":summary.resources,"edges":summary.edges,
        "elapsed_seconds":elapsed,"resource_rows_per_second":f64::from(rows)/elapsed.max(f64::EPSILON)}),
    )
}

fn fixture(page: u32, rows: u32, page_size: u32) -> CollectionPage {
    let start = page.saturating_mul(page_size);
    let end = start.saturating_add(page_size).min(rows);
    let resources:Vec<_>=(start..end).map(|index| Resource {
        id:format!("arn:aws:ec2:us-east-1:123456789012:instance/i-{index:09}"), account_id:"123456789012".into(),
        region:"us-east-1".into(), kind:"aws_ec2_instance".into(), name:Some(format!("fixture-{index}")),
        properties:json!({"instance_id":format!("i-{index:09}"),"instance_type":"m7i.large","state":"running","vpc_id":"vpc-bench"}),
        tags:BTreeMap::from([("Environment".into(),"synthetic".into())]), source:Source::Direct, observed_at:1,
    }).collect();
    let edges = resources
        .iter()
        .map(|resource| Edge {
            from_id: resource.id.clone(),
            to_id: "arn:aws:ec2:us-east-1:123456789012:vpc/vpc-bench".into(),
            relation: "IN_VPC".into(),
            source: Source::Direct,
            observed_at: 1,
            properties: json!({}),
        })
        .collect();
    CollectionPage {
        scope: CollectionScope {
            account_id: "123456789012".into(),
            region: "us-east-1".into(),
            collector: "ec2".into(),
            parent_id: None,
            source: Source::Direct,
        },
        page_number: page.saturating_add(1),
        next_token: (end < rows).then(|| format!("next-{end}")),
        resources,
        edges,
    }
}

#[tracing::instrument]
async fn network_cases() -> BenchResult<Vec<Value>> {
    stream::iter([1_usize, 8, 64])
        .map(Ok::<_, Box<dyn Error>>)
        .and_then(network_case)
        .try_collect()
        .await
}

#[tracing::instrument]
async fn network_case(concurrency: usize) -> BenchResult<Value> {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: concurrency,
            per_bucket_concurrency: concurrency,
            requests_per_second: 100_000,
            ..ExecutorConfig::default()
        },
        7,
    )?;
    let started = Instant::now();
    stream::iter(0..NETWORK_OPERATIONS)
        .map(|index| {
            let executor = &executor;
            async move {
                executor
                    .execute(&format!("mock-{index}"), || async {
                        tokio::time::sleep(Duration::from_millis(2)).await;
                        Ok::<_, ApiError>(())
                    })
                    .await
            }
        })
        .buffer_unordered(concurrency)
        .try_for_each(|()| std::future::ready(Ok(())))
        .await?;
    let elapsed = started.elapsed().as_secs_f64();
    Ok(
        json!({"global_concurrency":concurrency,"mock_latency_ms":2,"independent_buckets":NETWORK_OPERATIONS,
        "elapsed_seconds":elapsed,"operations_per_second":f64::from(NETWORK_OPERATIONS)/elapsed.max(f64::EPSILON),
        "stats":executor.stats()}),
    )
}
