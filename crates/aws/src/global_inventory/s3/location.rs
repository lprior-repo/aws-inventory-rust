use super::*;

pub(super) fn bucket_context(ctx: &Context, region: &str) -> Context {
    Context {
        config: ctx
            .config
            .to_builder()
            .region(Region::new(region.to_owned()))
            .build(),
        region: region.to_owned(),
        ..ctx.clone()
    }
}

pub(super) fn location_region(
    partition: &str,
    location: Option<&str>,
) -> Result<String, AwsScanError> {
    let region = match location {
        None | Some("") => "us-east-1",
        Some("EU") => "eu-west-1",
        Some(region) => region,
    };
    crate::validate_region(region)?;
    let partition_matches = match partition {
        "aws-cn" => region.starts_with("cn-"),
        "aws-us-gov" => region.starts_with("us-gov-"),
        "aws" => !region.starts_with("cn-") && !region.starts_with("us-gov-"),
        _ => false,
    };
    if !partition_matches {
        return Err(AwsScanError::InvalidData(
            "bucket region does not match the scanned partition".into(),
        ));
    }
    Ok(region.to_owned())
}

#[tracing::instrument(skip_all)]
pub(super) async fn resolve_location(
    ctx: &Context,
    client: &Client,
    bucket: &Bucket,
    name: &str,
) -> Result<String, AwsScanError> {
    if let Some(region) = bucket.bucket_region() {
        return location_region(&ctx.partition, Some(region));
    }
    let region = ctx
        .execute("s3", "read", || async {
            let result = client
                .head_bucket()
                .bucket(name)
                .expected_bucket_owner(&ctx.account)
                .send()
                .await;
            match result {
                Ok(output) => Ok(output.bucket_region().map(str::to_owned)),
                Err(error) => {
                    let region = error.raw_response().and_then(|response| {
                        location_header(
                            response.status().as_u16(),
                            response.headers().get("x-amz-bucket-region"),
                        )
                    });
                    match region {
                        Some(region) => Ok(Some(region)),
                        None => Err(sdk_error(error)),
                    }
                }
            }
        })
        .await?;
    let region = required(region.as_deref(), "HeadBucket region")?;
    location_region(&ctx.partition, Some(&region))
}

pub(super) fn location_header(status: u16, region: Option<&str>) -> Option<String> {
    if !matches!(status, 301 | 400 | 403 | 404) {
        return None;
    }
    region
        .filter(|region| crate::validate_region(region).is_ok())
        .map(str::to_owned)
}
