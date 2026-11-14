use std::collections::BTreeMap;

use aws_sdk_iam::{
    Client,
    types::{InstanceProfile, Policy, PolicyScopeType, Role, Tag},
};
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::{Edge, Resource, Source};
use serde_json::{Value, json};

use super::{
    CHILD_CONCURRENCY, SeenIds, checked_text, components, edge_set, ensure_active, marker,
    observed, own_arn, reference, single,
};
use crate::AwsScanError;
use crate::common::{
    Context, PageBody, collect_bounded, paginate, require_budget, required, sdk_error,
};

const ROLE: &str = "aws_iam_role";
const PROFILE: &str = "aws_iam_instance_profile";
const POLICY: &str = "aws_iam_policy";
const MAX_TRUST_BYTES: usize = 65_536;
const MAX_TRUST_STATEMENTS: usize = 128;
const MAX_PRINCIPALS: usize = 256;

#[tracing::instrument(skip_all)]
pub(super) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let client = Client::new(&ctx.config);
    let work: [BoxFuture<'_, Result<(), AwsScanError>>; 3] = [
        Box::pin(roles(ctx, &client)),
        Box::pin(profiles(ctx, &client)),
        Box::pin(policies(ctx, &client)),
    ];
    stream::iter(work)
        .buffer_unordered(3)
        .try_for_each(|()| async { Ok(()) })
        .await?;
    ensure_active(ctx)
}

mod roles;
use roles::*;

mod profiles;
use profiles::*;

mod policies;
use policies::*;

mod mapping;
use mapping::*;

mod trust;
use trust::*;

#[cfg(test)]
mod tests;
