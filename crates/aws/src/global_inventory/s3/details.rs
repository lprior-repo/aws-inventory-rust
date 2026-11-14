use super::*;

#[tracing::instrument(skip_all)]
pub(super) fn bucket_details<'a>(
    ctx: &'a Context,
    bucket: &'a Bucket,
    name: &'a str,
) -> BoxFuture<'a, Result<PageBody, AwsScanError>> {
    Box::pin(async move {
        let client = Client::new(&ctx.config);
        let mut work = Vec::with_capacity(SECTIONS.len());
        for key in SECTIONS {
            work.push(read_section(ctx, &client, name, key));
        }
        let values: Vec<(&'static str, Option<Value>)> =
            stream::iter(work).buffer_unordered(4).try_collect().await?;
        let tags = read_tags(ctx, &client, name).await?;
        map_bucket(ctx, bucket, Some(&ctx.region), &values, tags)
    })
}

pub(super) fn read_section<'a>(
    ctx: &'a Context,
    client: &'a Client,
    name: &'a str,
    key: &'static str,
) -> BoxFuture<'a, Result<(&'static str, Option<Value>), AwsScanError>> {
    Box::pin(async move {
        let collector = match key {
            "versioning" => "aws_s3_bucket_versioning",
            "encryption" => "aws_s3_bucket_encryption",
            "public_access_block" => "aws_s3_bucket_public_access_block",
            "policy_status" => "aws_s3_bucket_policy_status",
            "logging" => "aws_s3_bucket_logging",
            "replication" => "aws_s3_bucket_replication",
            "website" => "aws_s3_bucket_website",
            "ownership_controls" => "aws_s3_bucket_ownership_controls",
            "notifications" => "aws_s3_bucket_notifications",
            _ => {
                return Err(AwsScanError::InvalidOptions(
                    "unknown S3 metadata section".into(),
                ));
            }
        };
        let (_, value) = section(ctx, name, collector, || async {
            match key {
                "versioning" => versioning(ctx, client, name).await,
                "encryption" => encryption(ctx, client, name).await,
                "public_access_block" => public_access_block(ctx, client, name).await,
                "policy_status" => policy_status(ctx, client, name).await,
                "logging" => logging(ctx, client, name).await,
                "replication" => replication(ctx, client, name).await,
                "website" => website(ctx, client, name).await,
                "ownership_controls" => ownership(ctx, client, name).await,
                "notifications" => notifications(ctx, client, name).await,
                _ => Err(AwsScanError::InvalidOptions(
                    "unknown S3 metadata section".into(),
                )),
            }
        })
        .await?;
        Ok((key, value))
    })
}

pub(super) fn optional<T, E: ProvideErrorMetadata + ProvideErrorKind + Debug>(
    result: Result<T, SdkError<E>>,
    absent: &[&str],
) -> Result<Option<T>, ApiError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error)
            if error
                .as_service_error()
                .and_then(ProvideErrorMetadata::code)
                .is_some_and(|code| absent.contains(&code)) =>
        {
            Ok(None)
        }
        Err(error) => Err(sdk_error(error)),
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn read_tags(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    let (_, value) = section(ctx, name, "aws_s3_bucket_tags", || async {
        let output = ctx
            .execute("s3", "read", || async {
                optional(
                    client
                        .get_bucket_tagging()
                        .bucket(name)
                        .expected_bucket_owner(&ctx.account)
                        .send()
                        .await,
                    &["NoSuchTagSet"],
                )
            })
            .await?;
        let tags = match output {
            None => BTreeMap::new(),
            Some(output) => {
                components(output.tag_set().len(), "S3 tags")?;
                output
                    .tag_set()
                    .iter()
                    .map(|tag| {
                        Ok((
                            checked_text(tag.key(), "S3 tag key")?,
                            tag.value().to_owned(),
                        ))
                    })
                    .collect::<Result<BTreeMap<_, _>, AwsScanError>>()?
            }
        };
        Ok(json!(tags))
    })
    .await?;
    value
        .map(|value| {
            serde_json::from_value(value)
                .map_err(|_| AwsScanError::InvalidData("invalid mapped bucket tags".into()))
        })
        .transpose()
}
