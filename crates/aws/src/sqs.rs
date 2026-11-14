use std::collections::BTreeMap;

use aws_sdk_sqs::{
    Client,
    operation::{get_queue_attributes::GetQueueAttributesOutput, list_queues::ListQueuesOutput},
    types::QueueAttributeName,
};
use inventory_core::Source;

use crate::{
    AwsScanError,
    common::{
        Context, PageBody, collect_bounded, error_category, paginate, require_budget, sdk_error,
    },
};

mod queue;
use queue::{QueueObservation, QueueUrl, map_queue, parse_queue_url};

const LIST_QUEUES_MAX_RESULTS: i32 = 1_000;
const ENRICHMENT_CONCURRENCY: usize = 16;

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope("aws_sqs_queue", None, Source::Direct),
        |token, budget| {
            let client = &client;
            async move {
                let output = ctx
                    .execute("sqs", "control_read", || async {
                        client
                            .list_queues()
                            .max_results(list_page_size(ctx.page_size))
                            .set_next_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.queue_urls().len(), budget)?;
                let observations =
                    collect_bounded(output.queue_urls().iter(), ENRICHMENT_CONCURRENCY, |url| {
                        enrich(ctx, client, url)
                    })
                    .await?;
                Ok(page_body(
                    &output,
                    observations.into_iter().flatten().collect(),
                ))
            }
        },
    )
    .await
}

fn list_page_size(page_size: i32) -> i32 {
    page_size.clamp(1, LIST_QUEUES_MAX_RESULTS)
}

fn page_body(output: &ListQueuesOutput, observations: Vec<QueueObservation>) -> PageBody {
    let mut body = PageBody {
        next_token: output.next_token().map(str::to_owned),
        listed_count: output.queue_urls().len(),
        ..PageBody::default()
    };
    for observation in observations {
        body.resources.extend(observation.resources);
        body.edges.extend(observation.edges);
    }
    body
}

#[tracing::instrument(skip_all)]
async fn enrich(
    ctx: &Context,
    client: &Client,
    url: &str,
) -> Result<Option<QueueObservation>, AwsScanError> {
    let queue = match parse_queue_url(url) {
        Ok(queue) => queue,
        Err(message) => {
            return fail(
                ctx,
                "aws_sqs_queue",
                None,
                AwsScanError::InvalidData(message),
            )
            .await;
        }
    };
    let output = match queue_attributes(ctx, client, url, &queue).await? {
        Some(output) => output,
        None => return Ok(None),
    };
    let Some(attributes) = output.attributes() else {
        return fail(
            ctx,
            "aws_sqs_queue_attributes",
            Some(&queue),
            AwsScanError::InvalidData("GetQueueAttributes lacks attributes".into()),
        )
        .await;
    };
    let tags = queue_tags(ctx, client, url, &queue).await?;
    match map_queue(ctx, url, &queue, attributes, tags) {
        Ok(observation) => Ok(Some(observation)),
        Err(error) => fail(ctx, "aws_sqs_queue", Some(&queue), error).await,
    }
}

async fn queue_attributes(
    ctx: &Context,
    client: &Client,
    url: &str,
    queue: &QueueUrl<'_>,
) -> Result<Option<GetQueueAttributesOutput>, AwsScanError> {
    let result = ctx
        .execute("sqs", "control_read", || async {
            client
                .get_queue_attributes()
                .queue_url(url)
                .set_attribute_names(Some(attribute_names()))
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match result {
        Ok(output) => Ok(Some(output)),
        Err(error) => {
            fail(
                ctx,
                "aws_sqs_queue_attributes",
                Some(queue),
                AwsScanError::Api(error),
            )
            .await?;
            Ok(None)
        }
    }
}

async fn queue_tags(
    ctx: &Context,
    client: &Client,
    url: &str,
    queue: &QueueUrl<'_>,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    let result = ctx
        .execute("sqs", "control_read", || async {
            client
                .list_queue_tags()
                .queue_url(url)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match result {
        Ok(output) => Ok(Some(
            output
                .tags()
                .map(|tags| {
                    tags.iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect()
                })
                .unwrap_or_default(),
        )),
        Err(error) => {
            fail(
                ctx,
                "aws_sqs_queue_tags",
                Some(queue),
                AwsScanError::Api(error),
            )
            .await?;
            Ok(None)
        }
    }
}

async fn fail(
    ctx: &Context,
    collector: &str,
    queue: Option<&QueueUrl<'_>>,
    error: AwsScanError,
) -> Result<Option<QueueObservation>, AwsScanError> {
    ctx.failure(
        &ctx.scope(collector, queue.map(|queue| queue.name), Source::Direct),
        error_category(&error),
        error.to_string(),
    )
    .await?;
    Ok(None)
}

fn attribute_names() -> Vec<QueueAttributeName> {
    vec![
        QueueAttributeName::QueueArn,
        QueueAttributeName::ApproximateNumberOfMessages,
        QueueAttributeName::ApproximateNumberOfMessagesNotVisible,
        QueueAttributeName::VisibilityTimeout,
        QueueAttributeName::CreatedTimestamp,
        QueueAttributeName::FifoQueue,
        QueueAttributeName::KmsMasterKeyId,
        QueueAttributeName::SqsManagedSseEnabled,
        QueueAttributeName::RedrivePolicy,
    ]
}

#[cfg(test)]
mod tests;
