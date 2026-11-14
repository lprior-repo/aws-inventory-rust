use std::collections::{BTreeMap, BTreeSet};

use aws_sdk_sns::{
    Client,
    operation::{
        get_topic_attributes::GetTopicAttributesOutput,
        list_subscriptions::ListSubscriptionsOutput, list_topics::ListTopicsOutput,
    },
    types::Topic,
};
use inventory_core::{Resource, Source};

use crate::{
    AwsScanError,
    common::{
        Context, PageBody, collect_bounded, error_category, paginate, require_budget, sdk_error,
    },
};

mod arn;
mod subscription;
mod topic;
use subscription::{SubscriptionObservation, map_subscription};
use topic::{map_topic, tags_from};

const ENRICHMENT_CONCURRENCY: usize = 16;

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    tokio::try_join!(topics(ctx, &client), subscriptions(ctx, &client))?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn topics(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_sns_topic", None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("sns", "control_read", || async {
                    client
                        .list_topics()
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.topics().len(), budget)?;
            let resources =
                collect_bounded(output.topics().iter(), ENRICHMENT_CONCURRENCY, |topic| {
                    enrich_topic(ctx, client, topic)
                })
                .await?;
            Ok(topic_page_body(
                &output,
                resources.into_iter().flatten().collect(),
            ))
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn subscriptions(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_sns_subscription", None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("sns", "control_read", || async {
                    client
                        .list_subscriptions()
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.subscriptions().len(), budget)?;
            let mut observations = Vec::new();
            for item in output.subscriptions() {
                match map_subscription(ctx, item) {
                    Ok(Some(observation)) => observations.push(observation),
                    Ok(None) => {}
                    Err(error) => {
                        failure(ctx, "aws_sns_subscription", None, &error).await?;
                    }
                }
            }
            Ok(subscription_page_body(
                &output,
                dedupe_subscriptions(observations),
            ))
        },
    )
    .await
}

fn topic_page_body(output: &ListTopicsOutput, resources: Vec<Resource>) -> PageBody {
    PageBody {
        resources,
        next_token: output.next_token().map(str::to_owned),
        listed_count: output.topics().len(),
        ..PageBody::default()
    }
}

fn dedupe_subscriptions(
    observations: Vec<SubscriptionObservation>,
) -> Vec<SubscriptionObservation> {
    let mut seen = BTreeSet::new();
    observations
        .into_iter()
        .filter(|observation| seen.insert(observation.resource.id.clone()))
        .collect()
}

fn subscription_page_body(
    output: &ListSubscriptionsOutput,
    observations: Vec<SubscriptionObservation>,
) -> PageBody {
    let mut body = PageBody {
        next_token: output.next_token().map(str::to_owned),
        listed_count: output.subscriptions().len(),
        ..PageBody::default()
    };
    for observation in observations {
        body.resources.push(observation.resource);
        if let Some(edge) = observation.edge {
            body.edges.push(edge);
        }
    }
    body
}

async fn enrich_topic(
    ctx: &Context,
    client: &Client,
    topic: &Topic,
) -> Result<Option<Resource>, AwsScanError> {
    let Some(arn) = topic.topic_arn().filter(|value| !value.is_empty()) else {
        return fail(
            ctx,
            "aws_sns_topic",
            None,
            AwsScanError::InvalidData("ListTopics lacks TopicArn".into()),
        )
        .await;
    };
    let parts = match arn::split_sns_arn(arn) {
        Ok(parts) => parts,
        Err(message) => {
            return fail(
                ctx,
                "aws_sns_topic",
                None,
                AwsScanError::InvalidData(message),
            )
            .await;
        }
    };
    if !arn::is_in_scope(&parts, ctx) {
        return fail(
            ctx,
            "aws_sns_topic",
            None,
            AwsScanError::InvalidData("SNS topic ARN is outside the scan scope".into()),
        )
        .await;
    }
    if let Err(message) = arn::validate_topic_name(parts.resource) {
        return fail(
            ctx,
            "aws_sns_topic",
            None,
            AwsScanError::InvalidData(message),
        )
        .await;
    }
    let output = match topic_attributes(ctx, client, arn).await? {
        Some(output) => output,
        None => return Ok(None),
    };
    let Some(attributes) = output.attributes() else {
        return fail(
            ctx,
            "aws_sns_topic_attributes",
            Some(parts.resource),
            AwsScanError::InvalidData("GetTopicAttributes lacks attributes".into()),
        )
        .await;
    };
    let tags = topic_tags(ctx, client, arn, parts.resource).await?;
    match map_topic(ctx, arn, attributes, tags) {
        Ok(resource) => Ok(Some(resource)),
        Err(error) => fail(ctx, "aws_sns_topic", Some(parts.resource), error).await,
    }
}

async fn topic_attributes(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<GetTopicAttributesOutput>, AwsScanError> {
    let result = ctx
        .execute("sns", "control_read", || async {
            client
                .get_topic_attributes()
                .topic_arn(arn)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match result {
        Ok(output) => Ok(Some(output)),
        Err(error) => {
            let error = AwsScanError::Api(error);
            let parent = arn::split_sns_arn(arn).ok().map(|parts| parts.resource);
            failure(ctx, "aws_sns_topic_attributes", parent, &error).await?;
            Ok(None)
        }
    }
}

async fn topic_tags(
    ctx: &Context,
    client: &Client,
    arn: &str,
    topic_name: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    let result = ctx
        .execute("sns", "control_read", || async {
            client
                .list_tags_for_resource()
                .resource_arn(arn)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    match result {
        Ok(output) => Ok(Some(tags_from(output.tags()))),
        Err(error) => {
            let error = AwsScanError::Api(error);
            failure(ctx, "aws_sns_topic_tags", Some(topic_name), &error).await?;
            Ok(None)
        }
    }
}

async fn fail(
    ctx: &Context,
    collector: &str,
    parent: Option<&str>,
    error: AwsScanError,
) -> Result<Option<Resource>, AwsScanError> {
    failure(ctx, collector, parent, &error).await?;
    Ok(None)
}

async fn failure(
    ctx: &Context,
    collector: &str,
    parent: Option<&str>,
    error: &AwsScanError,
) -> Result<(), AwsScanError> {
    ctx.failure(
        &ctx.scope(collector, parent, Source::Direct),
        error_category(error),
        error.to_string(),
    )
    .await
}

#[cfg(test)]
mod tests;
