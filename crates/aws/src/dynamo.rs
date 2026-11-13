use std::collections::BTreeMap;

use aws_sdk_dynamodb::Client;
use inventory_core::{Resource, Source};
use serde_json::json;

use crate::{
    AwsScanError,
    common::{
        Context, PageBody, collect_bounded, error_category, paginate, require_budget, required,
        sdk_error,
    },
};

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope("aws_dynamodb_table", None, Source::Direct),
        |token, budget| {
            let client = &client;
            async move {
                let output = ctx
                    .execute("dynamodb", "control_read", || async {
                        client
                            .list_tables()
                            .limit(ctx.page_size.clamp(1, 100))
                            .set_exclusive_start_table_name(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.table_names().len(), budget)?;
                let resources = collect_bounded(output.table_names().iter(), 16, |name| {
                    describe(ctx, client, name)
                })
                .await?
                .into_iter()
                .flatten()
                .collect();
                Ok(PageBody {
                    resources,
                    listed_count: output.table_names().len(),
                    next_token: output.last_evaluated_table_name().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(table_name = name))]
async fn describe(
    ctx: &Context,
    client: &Client,
    name: &str,
) -> Result<Option<Resource>, AwsScanError> {
    let output = ctx
        .execute("dynamodb", "control_read", || async {
            client
                .describe_table()
                .table_name(name)
                .send()
                .await
                .map_err(sdk_error)
        })
        .await;
    let output = match output {
        Ok(output) => output,
        Err(error) => {
            let error = AwsScanError::Api(error);
            ctx.failure(
                &ctx.scope("aws_dynamodb_table_description", Some(name), Source::Direct),
                error_category(&error),
                error.to_string(),
            )
            .await?;
            return Ok(None);
        }
    };
    let table = output
        .table()
        .ok_or_else(|| AwsScanError::InvalidData("DescribeTable lacks table".into()))?;
    let arn = required(table.table_arn(), "DynamoDB table ARN")?;
    let resource = ctx.resource("aws_dynamodb_table", arn, Some(name.to_owned()), json!({
        "table_name": table.table_name(), "table_id": table.table_id(),
        "table_status": table.table_status().map(|value| value.as_str()), "item_count": table.item_count(),
        "table_size_bytes": table.table_size_bytes(),
        "key_schema": table.key_schema().iter().map(|key| json!({"attribute_name":key.attribute_name(),"key_type":key.key_type().as_str()})).collect::<Vec<_>>(),
        "attribute_definitions": table.attribute_definitions().iter().map(|attribute| json!({"attribute_name":attribute.attribute_name(),"attribute_type":attribute.attribute_type().as_str()})).collect::<Vec<_>>(),
        "billing_mode": table.billing_mode_summary().and_then(|mode|mode.billing_mode()).map(|value|value.as_str()),
        "deletion_protection_enabled": table.deletion_protection_enabled(),
        "read_capacity_units": table.provisioned_throughput().and_then(|value|value.read_capacity_units()),
        "write_capacity_units": table.provisioned_throughput().and_then(|value|value.write_capacity_units()),
        "stream_arn": table.latest_stream_arn(), "tags_collected": false,
    }), BTreeMap::new())?;
    Ok(Some(resource))
}
