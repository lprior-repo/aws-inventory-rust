//! records responsibilities for route53.
use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn records(ctx: &Context, client: &Client, id: &str) -> Result<(), AwsScanError> {
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_route53_record_list", Some(id), Source::Direct),
        |token, budget| {
            let seen = &seen;
            async move {
                ensure_active(ctx)?;
                let cursor = RecordCursor::decode(token.as_deref())?;
                let output = ctx
                    .execute("route53", "read", || async {
                        client
                            .list_resource_record_sets()
                            .hosted_zone_id(id)
                            .max_items(ctx.page_size.clamp(1, 300))
                            .set_start_record_name(cursor.as_ref().map(|v| v.name.clone()))
                            .set_start_record_type(
                                cursor
                                    .as_ref()
                                    .map(|v| RrType::from(v.record_type.as_str())),
                            )
                            .set_start_record_identifier(
                                cursor.as_ref().and_then(|v| v.identifier.clone()),
                            )
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.resource_record_sets().len(), budget)?;
                let identities = output
                    .resource_record_sets()
                    .iter()
                    .map(|record| record_identity(ctx, id, record))
                    .collect::<Result<Vec<_>, _>>()?;
                let count = seen.record(identities.iter().map(String::as_str), budget)?;
                let mapped = collect_bounded(
                    output.resource_record_sets().iter().zip(identities.iter()),
                    CHILD_CONCURRENCY,
                    |(record, identity)| map_record_scoped(ctx, id, identity, record),
                )
                .await?;
                let mut body = PageBody {
                    next_token: next_record_cursor(&output)?,
                    listed_count: count,
                    ..PageBody::default()
                };
                for (resource, edges) in mapped.into_iter().flatten() {
                    body.resources.push(resource);
                    body.edges.extend(edges);
                }
                Ok(body)
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn map_record_scoped(
    ctx: &Context,
    zone: &str,
    identity: &str,
    record: &ResourceRecordSet,
) -> Result<Option<(Resource, Vec<Edge>)>, AwsScanError> {
    ensure_active(ctx)?;
    match map_record(ctx, zone, identity, record) {
        Ok(mapped) => Ok(Some(mapped)),
        Err(error) => {
            ctx.failure(
                &ctx.scope(RECORD, Some(identity), Source::Direct),
                error_category(&error),
                error.to_string(),
            )
            .await?;
            Ok(None)
        }
    }
}
