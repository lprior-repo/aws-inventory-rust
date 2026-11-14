use super::*;

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    collect_with(
        ctx,
        |token| {
            let client = &client;
            async move {
                ctx.execute("ecs", "read", || async {
                    client
                        .list_task_definitions()
                        .status(TaskDefinitionStatus::Active)
                        .sort(SortOrder::Asc)
                        .max_results(ctx.page_size.clamp(1, 100))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await
            }
        },
        |arn: String| {
            let client = &client;
            async move {
                ctx.execute("ecs", "read", || async {
                    client
                        .describe_task_definition()
                        .task_definition(arn.clone())
                        .include(TaskDefinitionField::Tags)
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await
            }
        },
    )
    .await
}

pub(super) fn remember_list_page(
    seen: &Mutex<ListedIdentities>,
    arns: &[String],
    budget: usize,
) -> Result<(), AwsScanError> {
    require_budget(arns.len(), budget)?;
    if arns.len() > 100 {
        return Err(AwsScanError::InvalidData(
            "ListTaskDefinitions exceeded its 100-result API page limit".into(),
        ));
    }
    let mut seen = seen.lock().map_err(|_| {
        AwsScanError::InvalidData("task-definition identity ledger was poisoned".into())
    })?;
    if seen.arns.len().saturating_add(arns.len()) > MAX_CHILDREN {
        return Err(AwsScanError::ChildLimit);
    }
    let mut page_ids = BTreeSet::new();
    let mut bytes = seen.bytes;
    for arn in arns {
        definition_identity(arn)?;
        if seen.arns.contains(arn) || !page_ids.insert(arn.as_str()) {
            return Err(AwsScanError::InvalidData(
                "ListTaskDefinitions repeated a revision ARN".into(),
            ));
        }
        bytes = bytes.checked_add(arn.len()).ok_or_else(|| {
            AwsScanError::InvalidData("task-definition identity byte count overflow".into())
        })?;
        if bytes > MAX_IDENTITY_BYTES {
            return Err(AwsScanError::InvalidData(
                "task-definition identity history exceeds 8 MiB".into(),
            ));
        }
    }
    seen.arns.extend(arns.iter().cloned());
    seen.bytes = bytes;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn collect_with<L, LFut, D, DFut>(
    ctx: &Context,
    list: L,
    describe: D,
) -> Result<(), AwsScanError>
where
    L: Fn(Option<String>) -> LFut + Send + Sync,
    LFut: Future<Output = Result<ListTaskDefinitionsOutput, ApiError>> + Send,
    D: Fn(String) -> DFut + Send + Sync,
    DFut: Future<Output = Result<DescribeTaskDefinitionOutput, ApiError>> + Send,
{
    let seen = Mutex::new(ListedIdentities::default());
    paginate(
        ctx,
        ctx.scope(LIST_SCOPE, Some("status=ACTIVE"), Source::Direct),
        |token, budget| {
            let list = &list;
            let describe = &describe;
            let seen = &seen;
            async move {
                let output = list(token).await?;
                remember_list_page(seen, output.task_definition_arns(), budget)?;
                collect_bounded(
                    output.task_definition_arns().iter().cloned(),
                    DESCRIBE_CONCURRENCY,
                    |arn| async move { describe_with(ctx, &arn, || describe(arn.clone())).await },
                )
                .await?;
                Ok(PageBody {
                    listed_count: output.task_definition_arns().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(task_definition_arn = arn))]
pub(super) async fn describe_with<F, Fut>(
    ctx: &Context,
    arn: &str,
    fetch: F,
) -> Result<(), AwsScanError>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = Result<DescribeTaskDefinitionOutput, ApiError>> + Send,
{
    paginate(
        ctx,
        ctx.scope(DEFINITION_KIND, Some(arn), Source::Direct),
        |token, budget| {
            let fetch = &fetch;
            async move {
                if token.is_some() {
                    return Err(AwsScanError::InvalidData(
                        "DescribeTaskDefinition is not paginated".into(),
                    ));
                }
                require_budget(1, budget)?;
                let output = fetch().await?;
                map_definition(ctx, arn, &output)
            }
        },
    )
    .await
}
