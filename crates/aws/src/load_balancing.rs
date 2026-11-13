//! ELBv2 topology and target-registration evidence. Every child has its own scope.

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

use aws_sdk_elasticloadbalancingv2::{
    Client,
    operation::describe_target_health::DescribeTargetHealthOutput,
    types::{
        Action, AvailabilityZone, Listener, LoadBalancer, Rule, RuleCondition, RuleTransform,
        TargetDescription, TargetGroup, TargetHealthDescription,
    },
};
use inventory_core::{Edge, Resource, Source};
use inventory_engine::limiter::ApiError;
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{Context, PageBody, collect_bounded, paginate, require_budget, required, sdk_error},
};

const CHILD_CONCURRENCY: usize = 4;
const MAX_NESTED_ITEMS: usize = 1_024;

#[cfg(test)]
mod tests;

mod collection;
pub(crate) use collection::collect;
mod pages;
use pages::{check_nested, map_listener, map_load_balancer, map_page};
mod targets;
use targets::{map_target_group, map_target_health};
mod rules;
use rules::{map_actions, map_rule};

#[cfg(test)]
use targets::returned_target_arn;

#[cfg(test)]
use collection::{application_rules, target_health_with};
