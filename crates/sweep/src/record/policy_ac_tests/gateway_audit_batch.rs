mod native_fixture;
mod owned_roots;
mod rest_position;
mod consumption;
mod wire;
use consumption::consume_family;
use wire::assert_wire;

use native_fixture::{child_body, page_keys};
use owned_roots::filtered_roots;

use super::{TestResult, compile, cursor::Cursor, decode, resource_key};
use crate::{
    execution::page::{child, consumer_index::ConsumerIndex},
    input, record, request,
};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ScopeKey, ServiceId},
    record::{Record, ValueRef},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use serde_json::{Value, json};
use std::collections::BTreeSet;

struct ChildCase {
    operation: &'static str,
    kind: &'static str,
    items: &'static str,
    id: &'static str,
}

struct Family {
    service: &'static str,
    date: &'static str,
    root: &'static str,
    root_items: &'static str,
    root_key: &'static str,
    input_key: &'static str,
    token: &'static str,
    children: &'static [ChildCase],
}

fn text(value: &Record, fields: &catalog_tool::smithy::Registry, key: &str) -> TestResult<String> {
    match record::path_value(
        value.root(),
        &[fields.id(key).ok_or("modeled request member absent")?],
    )? {
        Some(ValueRef::Text(value)) => Ok(value.to_owned()),
        _ => Err(format!("required request text absent: {key}").into()),
    }
}

fn parent_id(family: &Family, suffix: &str) -> String {
    if family.service == "batch" {
        format!("arn:aws:batch:us-east-1:123456789012:job-queue/{suffix}")
    } else {
        format!("api-{suffix}")
    }
}


#[tokio::test]
async fn given_overlapping_rest_api_children_when_later_pages_resume_then_parent_qualified_resources_and_joins_survive()
-> TestResult {
    consume_family(&Family {
        service: "api-gateway",
        date: "2015-07-09",
        root: "GetRestApis",
        root_items: "item",
        root_key: "id",
        input_key: "restApiId",
        token: "position",
        children: &[
            ChildCase {
                operation: "GetAuthorizers",
                kind: "AWS::ApiGateway::Authorizer",
                items: "item",
                id: "id",
            },
            ChildCase {
                operation: "GetRequestValidators",
                kind: "AWS::ApiGateway::RequestValidator",
                items: "item",
                id: "id",
            },
            ChildCase {
                operation: "GetResources",
                kind: "AWS::ApiGateway::Resource",
                items: "item",
                id: "id",
            },
        ],
    })
    .await
}

#[tokio::test]
async fn given_overlapping_v2_api_children_when_later_pages_resume_then_routes_and_authorizers_keep_their_api()
-> TestResult {
    consume_family(&Family {
        service: "apigatewayv2",
        date: "2018-11-29",
        root: "GetApis",
        root_items: "items",
        root_key: "apiId",
        input_key: "ApiId",
        token: "nextToken",
        children: &[
            ChildCase {
                operation: "GetRoutes",
                kind: "AWS::ApiGatewayV2::Route",
                items: "items",
                id: "routeId",
            },
            ChildCase {
                operation: "GetAuthorizers",
                kind: "AWS::ApiGatewayV2::Authorizer",
                items: "items",
                id: "authorizerId",
            },
        ],
    })
    .await
}

#[tokio::test]
async fn given_two_queues_when_structured_all_name_filter_runs_then_every_status_and_later_quota_share_is_durable()
-> TestResult {
    consume_family(&Family {
        service: "batch",
        date: "2016-08-10",
        root: "DescribeJobQueues",
        root_items: "jobQueues",
        root_key: "jobQueueArn",
        input_key: "jobQueue",
        token: "nextToken",
        children: &[
            ChildCase {
                operation: "ListJobs",
                kind: "AWS::ApiOnly::BatchJob",
                items: "jobSummaryList",
                id: "jobId",
            },
            ChildCase {
                operation: "ListQuotaShares",
                kind: "AWS::Batch::QuotaShare",
                items: "quotaShares",
                id: "quotaShareArn",
            },
        ],
    })
    .await
}

#[tokio::test]
async fn given_custom_audit_resources_when_paginated_then_exact_custom_enums_and_canonical_rows_are_persisted()
-> TestResult {
    filtered_roots(
        "auditmanager",
        "2017-07-25",
        &[
            (
                "ListAssessmentFrameworks",
                "frameworkMetadataList",
                "arn",
                "frameworkType",
                "Custom",
                "AWS::AuditManager::AssessmentFramework",
            ),
            (
                "ListControls",
                "controlMetadataList",
                "arn",
                "controlType",
                "Custom",
                "AWS::ApiOnly::AuditManagerControl",
            ),
        ],
    )
    .await
}

#[tokio::test]
async fn given_account_owned_automation_resources_when_paginated_then_serialized_account_enum_is_compiled_and_sent()
-> TestResult {
    filtered_roots(
        "bedrock-data-automation",
        "2023-07-26",
        &[
            (
                "ListBlueprints",
                "blueprints",
                "blueprintArn",
                "resourceOwner",
                "ACCOUNT",
                "AWS::Bedrock::Blueprint",
            ),
            (
                "ListDataAutomationProjects",
                "projects",
                "projectArn",
                "resourceOwner",
                "ACCOUNT",
                "AWS::Bedrock::DataAutomationProject",
            ),
        ],
    )
    .await
}
