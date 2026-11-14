use super::*;

#[tracing::instrument(skip_all, fields(input=%input.display()))]
pub async fn push_file_with_options(
    input: PathBuf,
    checkpoint: PathBuf,
    url: &str,
    options: PushOptions,
    cancellation: &CancellationToken,
) -> Result<PushSummary, GraphError> {
    let options = options.validate()?;
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(options.overall_timeout_ms))
        .ok_or_else(|| GraphError::Invalid("graph deadline overflow".into()))?;
    let control = Control {
        cancellation,
        deadline,
    };
    let plan = control.run(freeze_plan(&input)).await?;
    let client = redis::Client::open(url).map_err(|error| GraphError::Redis(redacted(&error)))?;
    let destination = format!(
        "{:x}",
        base16ct::HexDisplay(&Sha256::digest(url.as_bytes()))
    );

    let log = ProjectionLog::prepare(&checkpoint, destination);
    let mut projection = Projection {
        client,
        connection: None,
        log,
        plan,
        options,
        summary: PushSummary::default(),
        connections: AtomicU64::new(0),
    };
    let result = control
        .run(async {
            projection.log.initialize().await?;
            bind_plan(&projection.log, &projection.plan).await?;
            apply_plan(&input, &mut projection).await
        })
        .await;

    drop(projection.connection.take());
    let closed = tokio::time::timeout(Duration::from_secs(10), projection.log.pool.close()).await;
    match (result, closed) {
        (_, Err(_)) => Err(GraphError::Cleanup {
            source: std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "graph checkpoint pool did not close within its drain deadline",
            ),
        }),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Ok(())) => {
            projection.summary.reconnects = projection
                .connections
                .load(Ordering::Relaxed)
                .saturating_sub(1);
            Ok(projection.summary)
        }
    }
}

pub(super) async fn apply_plan(
    input: &Path,
    projection: &mut Projection,
) -> Result<(), GraphError> {
    let file = File::open(input).await?;
    stream::try_unfold(PlanReader::new(file), next_batch)
        .try_fold(projection, apply_batch)
        .await?;
    Ok(())
}
