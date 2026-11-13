//! Hosted zones and record sets. Record cursors preserve name, type and routing
//! identifier together. TXT/SPF and unknown RDATA are omitted, not interpreted.

use std::collections::BTreeMap;

use aws_sdk_route53::{
    Client,
    operation::{
        get_hosted_zone::GetHostedZoneOutput,
        list_resource_record_sets::ListResourceRecordSetsOutput,
    },
    types::{HostedZone, ResourceRecordSet, RrType, TagResourceType},
};
use inventory_core::{Edge, Resource, Source};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    CHILD_CONCURRENCY, SeenIds, checked_text, components, digest_identity, dns_reference, edge_set,
    ensure_active, marker, observed, reference, section, single,
};
use crate::{
    AwsScanError,
    common::{
        Context, PageBody, collect_bounded, error_category, paginate, require_budget, required,
        sdk_error,
    },
};

const ZONE: &str = "aws_route53_zone";
const RECORD: &str = "aws_route53_record";
const MAX_RECORD_VALUES: usize = 512;

#[tracing::instrument(skip_all)]
pub(super) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let client = Client::new(&ctx.config);
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_route53_zone_list", None, Source::Direct),
        |token, budget| {
            let seen = &seen;
            let client = &client;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("route53", "read", || async {
                        client
                            .list_hosted_zones()
                            .set_marker(token.clone())
                            .max_items(ctx.page_size.clamp(1, 100))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                let count =
                    seen.record(output.hosted_zones().iter().map(HostedZone::id), budget)?;
                let next_token = marker(output.is_truncated(), output.next_marker())?;
                collect_bounded(output.hosted_zones(), CHILD_CONCURRENCY, |zone| {
                    collect_zone(ctx, client, zone)
                })
                .await?;
                Ok(PageBody {
                    next_token,
                    listed_count: count,
                    ..PageBody::default()
                })
            }
        },
    )
    .await?;
    ensure_active(ctx)
}

mod cursor;
use cursor::*;

mod zones;
use zones::*;

mod records;
use records::*;

mod mapping;
use mapping::*;

#[cfg(test)]
mod tests;
